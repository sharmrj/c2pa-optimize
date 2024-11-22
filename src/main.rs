use magick_rust::{MagickWand, MagickError, magick_wand_genesis};
use std::{sync::Once, path::PathBuf};
use c2pa::{
    assertions::{c2pa_action, Action, Actions}, Reader,
    create_signer, Builder, ClaimGeneratorInfo, Ingredient, Relationship, SigningAlg,
};
use std::fs;

static START: Once = Once::new();
const GENERATOR: &str = "test_app/0.1";

fn main() -> Result<(), c2pa::Error> {
    let stream = std::fs::File::open("soup.png")?;
    let reader = Reader::from_stream("image/png", stream)?;
    let manifestJson = reader.json();
    match resize("soup.png") {
        Ok(bytes) => fs::write("opt-test.webp", bytes).expect("Uh oh"),
        Err(err) => println!("Err: {}", err)
    }
    let source = PathBuf::from("opt-test.webp");
    let dest   = PathBuf::from("opt-test-signed.webp");
    let mut parent = Ingredient::from_file(source.as_path())?;
    parent.set_relationship(Relationship::ParentOf);
    let actions = Actions::new().add_action(
        Action::new(c2pa_action::CONVERTED)
            .set_parameter("identifier", parent.instance_id().to_owned())?,
    );
    let mut builder = Builder::from_json(&manifestJson)?;
    builder
        .set_claim_generator_info(ClaimGeneratorInfo::new(GENERATOR))
        .add_assertion(Actions::LABEL, &actions)?;
    // sign and embed into the target file
    let signcert_path = "cert/es256.pub";
    let pkey_path = "cert/es256.pem";
    let signer = create_signer::from_files(signcert_path, pkey_path, SigningAlg::Es256, None)?;

    builder.sign_file(&*signer, &source, &dest);
    Ok(())
}

fn resize(filepath: &str) -> Result<Vec<u8>, MagickError> {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let mut wand = MagickWand::new();
    wand.read_image(filepath);
    wand.set_compression(magick_rust::CompressionType::WebP);
    wand.write_image_blob("webp")
}
