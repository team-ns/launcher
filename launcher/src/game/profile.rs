use crate::client::AuthInfo;
use jni::objects::{JObject, JValue};
use jni::JNIEnv;
use launcher_api::profile::Profile;
use path_slash::PathExt;
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(not(target_os = "windows"))]
const CLASS_PATH_SEPARATOR: &str = ":";
#[cfg(target_os = "windows")]
const CLASS_PATH_SEPARATOR: &str = ";";

pub trait ClientProfile {
    fn new(path: &str) -> Self;
    fn create_lib_string(&self, dir: &str) -> String;
    fn get_native_option(&self, dir: &str) -> String;
    fn create_args(
        &self,
        dir: &str,
        env: &JNIEnv,
        auth_info: AuthInfo,
        profile: &Profile,
    ) -> JValue;
    fn get_client_dir(&self, dir: &str) -> PathBuf;
}

impl ClientProfile for Profile {
    fn new(path: &str) -> Profile {
        serde_json::from_reader(File::open(path).unwrap()).unwrap()
    }

    fn create_lib_string(&self, dir: &str) -> String {
        let mut path = String::from("-Djava.class.path=");
        for library in &self.libraries {
            path += &Path::new(&[dir, "/libraries/", library, CLASS_PATH_SEPARATOR].join(""))
                .to_slash_lossy();
        }
        let class_path: Vec<_> = self
            .class_path
            .iter()
            .map(|s| self.get_client_dir(dir).join(&s).to_slash_lossy())
            .collect();
        path += &class_path.join(CLASS_PATH_SEPARATOR);
        path
    }

    fn get_native_option(&self, dir: &str) -> String {
        format!(
            "{}{}",
            "-Djava.library.path=",
            Path::new(dir)
                .join("natives")
                .join(&self.version)
                .to_slash_lossy()
        )
    }

    fn create_args(
        &self,
        dir: &str,
        env: &JNIEnv,
        auth_info: AuthInfo,
        profile: &Profile,
    ) -> JValue {
        let mut args = self.client_args.clone();
        args.push(String::from("--gameDir"));
        args.push(self.get_client_dir(dir).to_string_lossy().to_string());
        args.push(String::from("--assetsDir"));
        args.push(Path::new(dir).join(&self.assets_dir).to_slash_lossy());
        args.push(String::from("--assetIndex"));
        args.push(self.assets.to_string());
        args.push(String::from("--uuid"));
        args.push(auth_info.uuid);
        args.push(String::from("--accessToken"));
        args.push(auth_info.access_token);
        args.push(String::from("--username"));
        args.push(auth_info.username);
        args.push(String::from("--server"));
        args.push(profile.server_name.clone());
        args.push(String::from("--port"));
        args.push(profile.server_port.to_string());
        let array = env
            .new_object_array(
                args.len() as i32,
                env.find_class("java/lang/String").unwrap(),
                JObject::from(env.new_string("").unwrap()),
            )
            .unwrap();
        for (i, arg) in args.iter().enumerate() {
            env.set_object_array_element(
                array,
                i as i32,
                JObject::from(env.new_string(&arg).unwrap()),
            )
            .unwrap();
        }
        JValue::from(JObject::from(array))
    }

    fn get_client_dir(&self, dir: &str) -> PathBuf {
        Path::new(dir).join("profiles").join(&self.name)
    }
}
