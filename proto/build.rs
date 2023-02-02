use std::path::Path;

const DERIVE_SER_DER: &str = "#[derive(serde::Serialize, serde::Deserialize)]";
const DERIVE_SER: &str = "#[derive(serde::Serialize)]";
const DERIVE_DER: &str = "#[derive(serde::Deserialize)]";
const DERIVE_DEFAULT: &str = "#[serde(default)]";

fn main() {
    let protos: Vec<_> = glob::glob("**/*.proto")
        .unwrap()
        .flat_map(Result::ok)
        .collect();
    // let protos = ["auth/token/v1/token.proto", "user/sys/v1/sys.proto"];
    let proto_dir: &Path = ".".as_ref();

    tonic_build::configure()
        .derive_for("auth.token.v1.Token", vec![DERIVE_SER_DER, DERIVE_DEFAULT])
        .derive_for(
            "auth.token.v1.Payload",
            vec![DERIVE_SER_DER, DERIVE_DEFAULT],
        )
        .out_dir("src/gen")
        .compile(&protos, &[proto_dir])
        .unwrap();

    // prevent needing to rebuild if files (or deps) haven't changed
    println!("cargo:rerun-if-changed={}", proto_dir.to_str().unwrap());
}

trait BuilderExt {
    fn derive_for(self, typ: &str, attr: Vec<&str>) -> Self;
}

impl BuilderExt for tonic_build::Builder {
    fn derive_for(mut self, typ: &str, attr: Vec<&str>) -> Self {
        for ele in attr {
            self = self.type_attribute(typ, ele);
        }
        self
    }
}
