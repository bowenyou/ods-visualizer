use std::{io::{stdout, Write, Read}, collections::HashMap, fs::File, env};

use base64::{Engine, engine::general_purpose::STANDARD};
use celestia_rpc::{Client, HeaderClient, ShareClient};
use celestia_types::{nmt::Namespace, Share, ExtendedDataSquare};

use crossterm::{queue, style::{SetBackgroundColor, Color, Print, ResetColor}};
use serde::{Serialize, Deserialize};
use sha3::{Digest, Sha3_256};
use tokio::signal;

#[derive(Deserialize)]
struct Config {
    url: String,
    auth_key: String,
}

#[derive(Debug, Serialize)]
struct ODS {
    height: u64,
    cells: Vec<Vec<ODSCell>>
}

impl ODS {
    pub fn from_eds(eds: ExtendedDataSquare, height: u64) -> anyhow::Result<Self> {

        let mut ods = Vec::<Vec<ODSCell>>::new();
        let width = eds.square_width();
        let ods_width = width / 2;
        
        for i in 0..ods_width {
            ods.push(Vec::<ODSCell>::new());
            for j in 0..ods_width {
                let raw_share = eds.share(i, j)?;
                let share = Share::from_raw(raw_share)?;
                ods[i as usize].push(ODSCell::from_share(share)); 
            }
        }

        Ok(Self { height, cells: ods })
    }

    pub fn draw_grid(&self) {
        let mut stdout = stdout();
        let mut legend = HashMap::<String, (u8, u8, u8)>::new();

        queue!(stdout, ResetColor, Print("\nHeight: "), Print(self.height), Print("\n")).unwrap();

        for row in &self.cells {
            for cell in row {
                let (r, g, b) = cell.rgb;
                queue!(stdout, SetBackgroundColor(Color::Rgb { r, g, b }), Print("  "), ResetColor).unwrap(); 
                legend.insert(cell.id.clone(), cell.rgb);
            }
            queue!(stdout, Print("\n")).unwrap();
        }

        queue!(stdout, Print("\nLegend: \n")).unwrap();
        for (id, color) in legend {
            let (r, g, b) = color;
            queue!(stdout, SetBackgroundColor(Color::Rgb { r, g, b }), Print(id), ResetColor).unwrap();
            queue!(stdout, Print("\n")).unwrap();
        }

        stdout.flush().unwrap();

    }
}

#[derive(Debug, Serialize)]
struct ODSCell {
    id: String,
    rgb: (u8, u8, u8)
}

impl ODSCell {
    pub fn from_share(share: Share) -> Self {
        let namespace_id = share.namespace().id().to_vec();

        let mut hasher = Sha3_256::new();
        hasher.update(&namespace_id);
        let hash = hasher.finalize();

        let r = hash[0] as u8;
        let g = hash[1] as u8;
        let b = hash[2] as u8;

        let id = match share.namespace() {
            Namespace::TRANSACTION => "TRANSACTION".to_string(), 
            Namespace::PAY_FOR_BLOB => "PAY_FOR_BLOB".to_string(),
            Namespace::PRIMARY_RESERVED_PADDING => "PRIMARY_RESERVED_PADDING".to_string(),
            Namespace::MIN_SECONDARY_RESERVED => "MIN_SECONDARY_RESERVED".to_string(),
            Namespace::TAIL_PADDING => "TAIL_PADDING_NAMESPACE".to_string(),
            _ => {
                STANDARD.encode(namespace_id).to_string()
            }
        };

        Self {
            id,
            rgb: (r, g, b)
        }
    }
}

async fn receive_eds(config: &Config) -> anyhow::Result<()> {

    let ws_client = Client::new(&format!("ws://{}", config.url), Some(&config.auth_key)).await?;
    let mut subscription = ws_client.header_subscribe().await?;

    let rpc_client = Client::new(&format!("http://{}", config.url), Some(&config.auth_key)).await?;

    loop {
        if let Some(msg) = subscription.next().await {
            match msg {
                Ok(header) => {
                    let height = header.height().value();
                    let eds = rpc_client.share_get_eds(&header).await?;
                    let ods = ODS::from_eds(eds, height)?;
                    ods.draw_grid();
                },
                Err(e) => {
                    println!("Subscription error: {:?}", e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{

    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config = toml::from_str::<Config>(&contents)?;

    let args = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        let block_height = args[1].parse::<u64>()?;
        let rpc_client = Client::new(&format!("http://{}", config.url), Some(&config.auth_key)).await?;

        let header = rpc_client.header_get_by_height(block_height).await?;
        let eds = rpc_client.share_get_eds(&header).await?;
        let ods = ODS::from_eds(eds, block_height)?;
        ods.draw_grid();
    } else {
        tokio::spawn(async move {
            match receive_eds(&config).await {
                Err(e) => {
                    println!("websocket error: {:?}", e);
                },
                _ => {}
            }
        });
        
        signal::ctrl_c().await.expect("Failed to listen to ctrl+c signal");
        println!("received ctrl+c signal, exiting");
    }
    
    
    Ok(())
}
