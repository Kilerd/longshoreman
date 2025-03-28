use serde::de::DeserializeOwned;
use serde::Serialize;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::Mutex;
use std::fs;
use tokio::fs::File;

struct FsStruct<T: Serialize + DeserializeOwned> {
    path: String,
    data: Arc<Mutex<T>>,
}

unsafe impl<T: Serialize + DeserializeOwned + Send> Send for FsStruct<T> {}


impl<T: Serialize + DeserializeOwned> FsStruct<T> {
    pub async fn new(path: String) -> Self {
        let data = tokio::fs::read_to_string(&path).await.unwrap_or_else(|_| "{}".to_string());
        let data: T = serde_json::from_str(&data).expect("Failed to parse JSON");
        Self { 
            path,
            data: Arc::new(Mutex::new(data))
        }
    }
    pub fn lock(&self) -> impl std::ops::DerefMut<Target = T> + '_ {
        struct GuardWithSave<'a, T: Serialize + DeserializeOwned> {
            guard: std::sync::MutexGuard<'a, T>,
            fs: &'a FsStruct<T>,
        }

        impl<'a, T: Serialize + DeserializeOwned> Drop for GuardWithSave<'a, T> {
            fn drop(&mut self) {
                if let Ok(json) = serde_json::to_string_pretty(&*self.guard) {
                    let _ = fs::write(&self.fs.path, json);
                }
            }
        }

        impl<'a, T: Serialize + DeserializeOwned> std::ops::Deref for GuardWithSave<'a, T> {
            type Target = T;
            fn deref(&self) -> &T {
                &self.guard
            }
        }

        impl<'a, T: Serialize + DeserializeOwned> std::ops::DerefMut for GuardWithSave<'a, T> {
            fn deref_mut(&mut self) -> &mut T {
                &mut self.guard
            }
        }

        GuardWithSave {
            guard: self.data.lock().unwrap(),
            fs: self,
        }
    }
}
