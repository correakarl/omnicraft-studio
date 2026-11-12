// Gestor de Descargas Concurrentes (Máximo 2 simultáneas) en Rust nativo
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tokio::sync::Semaphore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub title: String,
    pub source: String,
    pub url: String,
    pub target_path: String,
    pub progress: f32,
    pub status: String,
}

pub struct DownloadManager {
    semaphore: Arc<Semaphore>,
    queue: Arc<Mutex<Vec<DownloadItem>>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(2)), // Concurrencia máxima 2
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_tasks(&self) -> Vec<DownloadItem> {
        self.queue.lock().unwrap().clone()
    }

    pub async fn enqueue_download(&self, item: DownloadItem) {
        {
            let mut q = self.queue.lock().unwrap();
            q.push(item.clone());
        }

        let sem = self.semaphore.clone();
        let queue_ref = self.queue.clone();
        let task_id = item.id.clone();
        let url = item.url.clone();
        let target_str = item.target_path.clone();

        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            
            // Actualizar estado a Descargando
            {
                let mut q = queue_ref.lock().unwrap();
                if let Some(t) = q.iter_mut().find(|x| x.id == task_id) {
                    t.status = "Descargando...".to_string();
                }
            }

            let client = reqwest::Client::new();
            let tmp_path = format!("{}.tmp", target_str);
            
            match client.get(&url).send().await {
                Ok(resp) => {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Some(parent) = PathBuf::from(&target_str).parent() {
                            std::fs::create_dir_all(parent).ok();
                        }
                        if std::fs::write(&tmp_path, &bytes).is_ok() {
                            std::fs::rename(&tmp_path, &target_str).ok();
                            let mut q = queue_ref.lock().unwrap();
                            if let Some(t) = q.iter_mut().find(|x| x.id == task_id) {
                                t.status = "Completado".to_string();
                                t.progress = 100.0;
                            }
                            return;
                        }
                    }
                }
                Err(_) => {}
            }

            // Si falló, limpiar rastro temporal
            std::fs::remove_file(&tmp_path).ok();
            let mut q = queue_ref.lock().unwrap();
            if let Some(t) = q.iter_mut().find(|x| x.id == task_id) {
                t.status = "Error".to_string();
            }
        });
    }

    pub fn cancel_task(&self, task_id: &str) {
        let mut q = self.queue.lock().unwrap();
        if let Some(t) = q.iter_mut().find(|x| x.id == task_id) {
            t.status = "Cancelado".to_string();
            let tmp = format!("{}.tmp", t.target_path);
            std::fs::remove_file(tmp).ok();
        }
    }
}
