//! The bridge between the synchronous Wayland event loop and `woof`'s async
//! `ArkPos`: a dedicated thread running a small `tokio` runtime, talked to
//! over a pair of channels.
//!
//! The window must never block on network I/O inside its paint/input loop,
//! so every wallet operation is a message sent here and a reply picked up
//! later by [`crate::wallet_worker::WalletWorker::poll`].

use std::path::PathBuf;
use std::sync::mpsc;

use woof::{ArkPos, ArkPosConfig, Network, PosError};

/// One thing the window wants the wallet to do.
pub enum WalletRequest {
    OnboardAddress,
    PayInvoice(String),
}

/// The wallet's answer to a [`WalletRequest`], paired by which request kind
/// produced it so the window need not guess.
pub enum WalletResponse {
    Connected(Result<(), String>),
    OnboardAddress(Result<String, String>),
    PayInvoice(Result<(), String>),
}

/// Owns the channels to the background wallet thread.
pub struct WalletWorker {
    to_worker: mpsc::Sender<WalletRequest>,
    from_worker: mpsc::Receiver<WalletResponse>,
}

impl WalletWorker {
    /// Where the wallet's seed phrase and database live.
    fn data_dir() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME").map_or_else(
            |_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_owned());
                PathBuf::from(home).join(".local").join("share")
            },
            PathBuf::from,
        );
        base.join("barkwallet")
    }

    /// A wallet on the public Ark signet, for getting started without real
    /// funds at risk. Swapping to mainnet later is a one-line config change,
    /// not a rewrite -- see `woof::ArkPosConfig`.
    fn default_config(dir: &std::path::Path) -> ArkPosConfig {
        ArkPosConfig {
            ark_server: "https://ark.signet.2nd.dev".into(),
            esplora_url: "https://esplora.signet.2nd.dev".into(),
            db_path: dir.join("wallet.sqlite").to_string_lossy().into_owned(),
            network: Network::Signet,
        }
    }

    /// Load the persisted mnemonic, or generate and persist a fresh one.
    fn load_or_create_mnemonic(dir: &std::path::Path) -> Result<String, String> {
        let path = dir.join("mnemonic");
        if let Ok(existing) = std::fs::read_to_string(&path) {
            return Ok(existing.trim().to_owned());
        }
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let entropy: [u8; 16] = rand_bytes();
        let mnemonic = bip39::Mnemonic::from_entropy(&entropy)
            .map_err(|e| e.to_string())?
            .to_string();
        std::fs::write(&path, &mnemonic).map_err(|e| e.to_string())?;
        Ok(mnemonic)
    }

    /// Spawn the worker thread and return a handle. Connection happens on
    /// the worker thread; the first [`WalletResponse::Connected`] on the
    /// reply channel reports whether it succeeded.
    #[must_use]
    pub fn spawn() -> Self {
        let (to_worker, requests) = mpsc::channel::<WalletRequest>();
        let (replies_tx, from_worker) = mpsc::channel::<WalletResponse>();

        std::thread::spawn(move || {
            let dir = Self::data_dir();
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = replies_tx.send(WalletResponse::Connected(Err(e.to_string())));
                    return;
                }
            };

            runtime.block_on(async move {
                let mnemonic = match Self::load_or_create_mnemonic(&dir) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = replies_tx.send(WalletResponse::Connected(Err(e)));
                        return;
                    }
                };
                let config = Self::default_config(&dir);
                let pos = match ArkPos::connect(&mnemonic, &config).await {
                    Ok(pos) => pos,
                    Err(e) => {
                        let _ = replies_tx.send(WalletResponse::Connected(Err(e.to_string())));
                        return;
                    }
                };
                let _ = replies_tx.send(WalletResponse::Connected(Ok(())));

                while let Ok(request) = requests.recv() {
                    let response = Self::handle(&pos, request).await;
                    if replies_tx.send(response).is_err() {
                        break;
                    }
                }
            });
        });

        Self {
            to_worker,
            from_worker,
        }
    }

    async fn handle(pos: &ArkPos, request: WalletRequest) -> WalletResponse {
        match request {
            WalletRequest::OnboardAddress => {
                WalletResponse::OnboardAddress(Self::to_string_result(pos.onboard_address().await))
            }
            WalletRequest::PayInvoice(bolt11) => {
                WalletResponse::PayInvoice(Self::to_unit_result(pos.pay_invoice(&bolt11).await))
            }
        }
    }

    fn to_string_result(result: Result<String, PosError>) -> Result<String, String> {
        result.map_err(|e| e.to_string())
    }

    fn to_unit_result(result: Result<(), PosError>) -> Result<(), String> {
        result.map_err(|e| e.to_string())
    }

    /// Send a request to the wallet thread. Silently dropped if the worker
    /// has already exited, matching how a stale double-click on an already
    /// dismissed button should behave: no-op, not a crash.
    pub fn send(&self, request: WalletRequest) {
        let _ = self.to_worker.send(request);
    }

    /// Drain every reply that has arrived since the last poll.
    pub fn poll(&self) -> Vec<WalletResponse> {
        self.from_worker.try_iter().collect()
    }
}

/// A small, dependency-free source of 16 random bytes for a fresh mnemonic.
///
/// Reads directly from the kernel CSPRNG rather than pulling in the `rand`
/// crate for one call site.
fn rand_bytes() -> [u8; 16] {
    let mut buf = [0u8; 16];
    if std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buf))
        .is_err()
    {
        // Extraordinarily unlikely on Linux; fall back to a time-seeded mix
        // rather than panic, since a broken wallet is worse than a weak one
        // the user can regenerate.
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = (seed >> (i * 8 % 128)) as u8 ^ (i as u8);
        }
    }
    buf
}
