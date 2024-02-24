mod bilibili;
mod bilivideo;
mod douyu;
mod fudujikiller;
mod huya;
mod mkv_header;
mod twitch;
mod youtube;

use crate::ipcmanager::IPCManager;
use crate::{config::ConfigManager, dmlive::DMLMessage, ipcmanager::DMLStream};
use anyhow::anyhow;
use anyhow::Result;
use async_channel::Sender;
use chrono::{Duration, NaiveTime};
use log::info;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ops::{BitXor, Not};
use std::rc::Rc;
use tokio::io::AsyncWriteExt;

const ASS_HEADER_TEXT: &'static str = r#"[Script Info]
; Script generated by dmlive 
; https://github.com/THMonster/Revda
Title: Danmaku file
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Sans,40,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,1,0,7,0,0,0,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;
const EMOJI_RE: &'static str = r#"[\x{1F300}-\x{1F5FF}|\x{1F1E6}-\x{1F1FF}|\x{2700}-\x{27BF}|\x{1F900}-\x{1F9FF}|\x{1F600}-\x{1F64F}|\x{1F680}-\x{1F6FF}|\x{2600}-\x{26FF}]"#;

#[derive(Clone, Debug)]
struct DanmakuChannel {
    length: usize,
    begin_pts: u64,
}
pub struct Danmaku {
    ipc_manager: Rc<IPCManager>,
    cm: Rc<ConfigManager>,
    show_nick: Cell<bool>,
    font_size: Cell<usize>,
    channel_num: Cell<usize>,
    ratio_scale: Cell<f64>,
    read_order: Cell<usize>,
    bili_video_cid: RefCell<String>,
    dchannels: RefCell<Vec<DanmakuChannel>>,
    fk: fudujikiller::FudujiKiller,
}

impl Danmaku {
    pub fn new(cm: Rc<ConfigManager>, im: Rc<IPCManager>, _mtx: Sender<DMLMessage>) -> Self {
        let font_size = (40.0 * cm.font_scale.get()) as usize;
        let ch = vec![
            DanmakuChannel {
                length: 0,
                begin_pts: 0
            };
            30
        ];
        Self {
            ipc_manager: im,
            cm,
            show_nick: Cell::new(false),
            font_size: Cell::new(font_size),
            channel_num: Cell::new((540.0 / font_size as f64).ceil() as usize),
            ratio_scale: Cell::new(1.0),
            read_order: Cell::new(0),
            fk: fudujikiller::FudujiKiller::new(),
            bili_video_cid: RefCell::new("".into()),
            dchannels: RefCell::new(ch),
        }
    }

    pub fn reset(&self) {
        for it in self.dchannels.borrow_mut().iter_mut() {
            it.length = 0;
            it.begin_pts = 0;
        }
        self.read_order.set(0)
    }

    pub async fn set_speed(&self, speed: u64) {
        if (1000..30000).contains(&speed) {
            self.cm.danmaku_speed.set(speed);
            let _ = self.cm.write_config().await;
        }
    }

    pub async fn set_font_size(&self, font_scale: f64) {
        if font_scale > 0.0 {
            self.font_size.set((40.0 * font_scale) as usize);
            self.channel_num.set((540.0 / self.font_size.get() as f64).ceil() as usize);
            self.cm.font_scale.set(font_scale);
            let _ = self.cm.write_config().await;
        }
    }

    pub async fn set_font_alpha(&self, font_alpha: f64) {
        if (0.0..=1.0).contains(&font_alpha) {
            self.cm.font_alpha.set(font_alpha);
            let _ = self.cm.write_config().await;
        }
    }

    pub async fn set_bili_video_cid(&self, cid: &str) {
        let mut bvc = self.bili_video_cid.borrow_mut();
        bvc.clear();
        bvc.push_str(cid);
    }

    pub async fn toggle_show_nick(&self) {
        self.show_nick.set(self.show_nick.get().bitxor(true));
    }

    fn get_avail_danmaku_channel(&self, c_pts: u64, len: usize) -> Option<usize> {
        let s = (1920.0 + len as f64) / self.cm.danmaku_speed.get() as f64;
        for (i, c) in self.dchannels.borrow_mut().iter_mut().enumerate() {
            if i >= self.channel_num.get() {
                break;
            }
            if c.length == 0 {
                c.length = len;
                c.begin_pts = c_pts;
                return Some(i);
            }
            if ((self.cm.danmaku_speed.get() as f64 - c_pts as f64 + c.begin_pts as f64) * s) > 1920.0 {
                continue;
            } else if ((c.length + 1920) as f64 * (c_pts as f64 - c.begin_pts as f64)
                / self.cm.danmaku_speed.get() as f64)
                < c.length as f64
            {
                continue;
            } else {
                c.length = len;
                c.begin_pts = c_pts;
                return Some(i);
            }
        }
        None
    }

    fn get_danmaku_display_length(&self, nick: &str, dm: &str) -> usize {
        let mut ascii_num = 0;
        let mut non_ascii_num = 0;
        for c in dm.chars() {
            if c.is_ascii() {
                ascii_num += 1;
            } else {
                non_ascii_num += 1;
            }
        }
        if self.show_nick.get() {
            for c in nick.chars() {
                if c.is_ascii() {
                    ascii_num += 1;
                } else {
                    non_ascii_num += 1;
                }
            }
            non_ascii_num += 1;
        }
        let fs = self.font_size.get();
        (((fs as f64 * 0.75 * non_ascii_num as f64) + (fs as f64 * 0.50 * ascii_num as f64)) * self.ratio_scale.get())
            .round() as usize
    }

    async fn launch_single_danmaku(
        &self, c: &str, n: &str, d: &str, c_pts: u64, socket: &mut Box<dyn DMLStream>,
    ) -> Result<()> {
        let mut out_of_channel = false;
        let mut f1 = || {
            n.trim().is_empty().not().then(|| {})?;
            let display_length = self.get_danmaku_display_length(n, d);
            self.get_avail_danmaku_channel(c_pts, display_length)
                .or_else(|| {
                    out_of_channel = true;
                    None
                })
                .map(|it| (it, display_length))
        };
        let cluster = match f1() {
            Some((avail_dc, display_length)) => {
                let ass = format!(
                    r"{4},0,Default,{5},0,0,0,,{{\alpha{0}\fs{7}\1c&{6}&\move(1920,{1},{2},{1})}}{8}{9}{3}",
                    format_args!("{:02x}", (self.cm.font_alpha.get() * 255_f64) as u8),
                    avail_dc * self.font_size.get(),
                    0 - display_length as isize,
                    &d,
                    self.read_order.get(),
                    &n,
                    format_args!("{}{}{}", &c[4..6], &c[2..4], &c[0..2]),
                    self.font_size.get(),
                    if self.show_nick.get() { n } else { "" },
                    if self.show_nick.get() { ": " } else { "" },
                )
                .into_bytes();
                mkv_header::DMKVCluster::new(ass, c_pts, self.cm.danmaku_speed.get())
            }
            None => {
                let ass = format!(
                    r"{},0,Default,dmlive-empty,20,20,2,,",
                    self.read_order.get()
                )
                .into_bytes();
                mkv_header::DMKVCluster::new(ass, c_pts, 1)
            }
        };
        self.read_order.set(self.read_order.get() + 1);
        cluster.write_to_socket(socket).await.map_err(|_| anyhow!("socket error"))?;
        out_of_channel.not().then(|| {}).ok_or_else(|| anyhow!("channels unavailable"))
    }

    pub async fn danmaku_client_task(&self, dtx: async_channel::Sender<(String, String, String)>) -> Result<()> {
        loop {
            match match self.cm.site {
                crate::config::Site::BiliLive => {
                    let b = bilibili::Bilibili::new();
                    b.run(&self.cm.room_url, dtx.clone()).await
                }
                crate::config::Site::BiliVideo => {
                    let b = bilivideo::Bilibili::new();
                    b.run(
                        format!(
                            "http://api.bilibili.com/x/v1/dm/list.so?oid={}",
                            self.bili_video_cid.borrow()
                        )
                        .as_str(),
                        dtx.clone(),
                    )
                    .await
                }
                crate::config::Site::DouyuLive => {
                    let b = douyu::Douyu::new();
                    b.run(&self.cm.room_url, dtx.clone()).await
                }
                crate::config::Site::HuyaLive => {
                    let b = huya::Huya::new();
                    b.run(&self.cm.room_url, dtx.clone()).await
                }
                crate::config::Site::TwitchLive => {
                    let b = twitch::Twitch::new();
                    b.run(&self.cm.room_url, dtx.clone()).await
                }
                crate::config::Site::YoutubeLive => {
                    let b = youtube::Youtube::new();
                    b.run(&self.cm.room_url, dtx.clone()).await
                }
            } {
                Ok(_) => {}
                Err(e) => {
                    info!("danmaku client error: {:?}", e);
                }
            };
            if dtx.is_closed() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        info!("danmaku client exited.");
        Ok(())
    }

    async fn launch_danmaku_task(&self, rx: async_channel::Receiver<(String, String, String)>) -> Result<()> {
        let now = std::time::Instant::now();
        let mut socket = self.ipc_manager.get_danmaku_socket().await?;
        let mut dm_queue = VecDeque::new();
        let emoji_re = regex::Regex::new(EMOJI_RE).unwrap();
        let empty_dm = ("".to_string(), "".to_string(), "".to_string());
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(200));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        socket.write_all(&mkv_header::get_mkv_header()).await?;
        let mut printed = false;
        'l1: loop {
            while let Ok(it) = rx.try_recv() {
                dm_queue.push_back(it);
            }
            let mut launch = true;
            while launch {
                let (co, ni, da) = dm_queue.get(0).ok_or_else(|| launch = false).unwrap_or(&empty_dm);
                if !da.is_empty() && !printed {
                    if !self.cm.quiet {
                        println!("[{}] {}", &ni, &da);
                        printed = true;
                    }
                    if !self.fk.dm_check(da) {
                        let _ = dm_queue.pop_front();
                        continue;
                    }
                }
                let da = emoji_re.replace_all(da, "[em]");
                let c_pts = now.elapsed().as_millis() as u64;
                match self.launch_single_danmaku(co, ni, &da, c_pts, &mut socket).await {
                    Ok(_) => {
                        let _ = dm_queue.pop_front();
                        printed = false;
                    }
                    Err(e) => {
                        info!("danmaku send error: {}", &e);
                        if e.to_string().contains("socket error") {
                            break 'l1;
                        } else {
                            launch = false;
                        }
                    }
                };
            }
            if self.read_order.get() > 70 {
                interval.tick().await;
            }
        }
        Ok(())
    }

    async fn launch_bvideo_danmaku_task(&self, rx: async_channel::Receiver<(String, String, String)>) -> Result<()> {
        let mut socket = self.ipc_manager.get_danmaku_socket().await?;
        let mut dm_map: BTreeMap<i64, (String, String, String)> = BTreeMap::new();
        while let Ok((c, n, d)) = rx.recv().await {
            let tmps: Vec<&str> = n.split(',').collect();
            dm_map.insert(
                (tmps[0].parse::<f64>().unwrap() * 1000.0) as i64,
                (c.to_string(), tmps[1].to_string(), d.to_string()),
            );
        }
        socket.write_all(ASS_HEADER_TEXT.as_bytes()).await?;
        for (k, (c, t, d)) in dm_map.into_iter() {
            info!("{}-{}-{}-{}", &k, &c, &t, &d);
            let t1 = NaiveTime::from_hms_opt(0, 0, 0).unwrap() + Duration::milliseconds(k);
            let t2 = t1 + Duration::milliseconds(self.cm.danmaku_speed.get() as i64);
            let mut t1_s = t1.format("%k:%M:%S%.3f").to_string();
            let mut t2_s = t2.format("%k:%M:%S%.3f").to_string();
            t1_s.remove(t1_s.len() - 1);
            t2_s.remove(t2_s.len() - 1);
            if t.trim().eq("4") {
                let ass = format!(
                    r#"Dialogue: 0,{4},{5},Default,,0,0,0,,{{\alpha{0}\fs{3}\1c&{2}&\an2}}{1}"#,
                    format_args!("{:02x}", (self.cm.font_alpha.get() * 255_f64) as u8),
                    &d,
                    format_args!("{}{}{}", &c[4..6], &c[2..4], &c[0..2]),
                    self.font_size.get(),
                    t1_s,
                    t2_s,
                );
                socket.write_all(ass.as_bytes()).await?;
                socket.write_all("\n".as_bytes()).await?;
            } else if t.trim().eq("5") {
                let ass = format!(
                    r#"Dialogue: 0,{4},{5},Default,,0,0,0,,{{\alpha{0}\fs{3}\1c&{2}&\an8}}{1}"#,
                    format_args!("{:02x}", (self.cm.font_alpha.get() * 255_f64) as u8),
                    &d,
                    format!("{}{}{}", &c[4..6], &c[2..4], &c[0..2]),
                    self.font_size.get(),
                    t1_s,
                    t2_s,
                );
                socket.write_all(ass.as_bytes()).await?;
                socket.write_all("\n".as_bytes()).await?;
            } else {
                let display_length = self.get_danmaku_display_length("", &d);
                let avail_dc = match self.get_avail_danmaku_channel(k as u64, display_length) {
                    Some(it) => it,
                    None => {
                        continue;
                    }
                };
                let ass = format!(
                    r#"Dialogue: 0,{4},{5},Default,,0,0,0,,{{\alpha{0}\fs{7}\1c&{6}&\move(1920,{1},{2},{1})}}{3}"#,
                    format_args!("{:02x}", (self.cm.font_alpha.get() * 255_f64) as u8),
                    avail_dc * self.font_size.get(),
                    0 - display_length as isize,
                    &d,
                    t1_s,
                    t2_s,
                    format!("{}{}{}", &c[4..6], &c[2..4], &c[0..2]),
                    self.font_size.get(),
                );
                socket.write_all(ass.as_bytes()).await?;
                socket.write_all("\n".as_bytes()).await?;
            }
        }
        Ok(())
    }

    pub async fn run_bilivideo(&self, ratio_scale: f64) -> Result<()> {
        // FIXME: a little hack here to prevent danmaku on --quiet set to true
        if self.cm.quiet { return Ok(()) }

        info!("ratio: {}", &ratio_scale);
        self.reset();
        self.ratio_scale.set(ratio_scale);
        let (dtx, drx) = async_channel::unbounded();
        let (dc_res, fbd_res) = tokio::join!(
            self.danmaku_client_task(dtx),
            self.launch_bvideo_danmaku_task(drx)
        );
        dc_res?;
        fbd_res?;
        info!("bilibili video danmaku exited");
        Ok(())
    }

    pub async fn run(&self, ratio_scale: f64, _start_pts: u64) -> Result<()> {
        // FIXME: a little hack here to prevent danmaku on --quiet set to true
        if self.cm.quiet { return Ok(()) }

        self.reset();
        self.ratio_scale.set(ratio_scale);
        let (dtx, drx) = async_channel::unbounded();
        tokio::select! {
            it = self.danmaku_client_task(dtx) => { it?; },
            it = self.launch_danmaku_task(drx) => { it?; },
        }
        info!("danmaku exited");
        Ok(())
    }
}
