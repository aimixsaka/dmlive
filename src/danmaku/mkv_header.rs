use anyhow::*;
use bytes::{BufMut, BytesMut};
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::ipcmanager::DMLStream;

pub fn get_mkv_header() -> Vec<u8> {
    vec![
        0x1a, 0x45, 0xdf, 0xa3, 0xa3, 0x42, 0x86, 0x81, 0x01, 0x42, 0xf7, 0x81, 0x01, 0x42, 0xf2, 0x81, 0x04, 0x42,
        0xf3, 0x81, 0x08, 0x42, 0x82, 0x88, 0x6d, 0x61, 0x74, 0x72, 0x6f, 0x73, 0x6b, 0x61, 0x42, 0x87, 0x81, 0x04,
        0x42, 0x85, 0x81, 0x02, 0x18, 0x53, 0x80, 0x67, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x11, 0x4d,
        0x9b, 0x74, 0xc1, 0xbf, 0x84, 0x35, 0x70, 0x23, 0xb2, 0x4d, 0xbb, 0x8b, 0x53, 0xab, 0x84, 0x15, 0x49, 0xa9,
        0x66, 0x53, 0xac, 0x81, 0xe5, 0x4d, 0xbb, 0x8c, 0x53, 0xab, 0x84, 0x16, 0x54, 0xae, 0x6b, 0x53, 0xac, 0x82,
        0x01, 0x35, 0x4d, 0xbb, 0x8c, 0x53, 0xab, 0x84, 0x12, 0x54, 0xc3, 0x67, 0x53, 0xac, 0x82, 0x04, 0x06, 0x4d,
        0xbb, 0x8c, 0x53, 0xab, 0x84, 0x1c, 0x53, 0xbb, 0x6b, 0x53, 0xac, 0x82, 0x04, 0xbe, 0xec, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x96, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x49, 0xa9, 0x66, 0xcb, 0xbf, 0x84,
        0x63, 0xab, 0xff, 0x6e, 0x2a, 0xd7, 0xb1, 0x83, 0x0f, 0x42, 0x40, 0x4d, 0x80, 0x8d, 0x4c, 0x61, 0x76, 0x66,
        0x35, 0x38, 0x2e, 0x32, 0x39, 0x2e, 0x31, 0x30, 0x30, 0x57, 0x41, 0x8d, 0x4c, 0x61, 0x76, 0x66, 0x35, 0x38,
        0x2e, 0x32, 0x39, 0x2e, 0x31, 0x30, 0x30, 0x73, 0xa4, 0x90, 0x07, 0x51, 0x6f, 0x2f, 0x07, 0x62, 0x01, 0xf2,
        0xd0, 0x3c, 0x06, 0xd9, 0x54, 0x7e, 0x86, 0x53, 0x44, 0x89, 0x88, 0x40, 0xb3, 0x88, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x16, 0x54, 0xae, 0x6b, 0x42, 0xcb, 0xbf, 0x84, 0xba, 0x3c, 0x54, 0x88, 0xae, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x02, 0xbc, 0xd7, 0x81, 0x01, 0x73, 0xc5, 0x81, 0x01, 0x9c, 0x81, 0x00, 0x22, 0xb5, 0x9c, 0x83,
        0x75, 0x6e, 0x64, 0x86, 0x8a, 0x53, 0x5f, 0x54, 0x45, 0x58, 0x54, 0x2f, 0x41, 0x53, 0x53, 0x83, 0x81, 0x11,
        0x63, 0xa2, 0x42, 0x98, 0x5b, 0x53, 0x63, 0x72, 0x69, 0x70, 0x74, 0x20, 0x49, 0x6e, 0x66, 0x6f, 0x5d, 0x0a,
        0x3b, 0x20, 0x53, 0x63, 0x72, 0x69, 0x70, 0x74, 0x20, 0x67, 0x65, 0x6e, 0x65, 0x72, 0x61, 0x74, 0x65, 0x64,
        0x20, 0x62, 0x79, 0x20, 0x51, 0x4c, 0x69, 0x76, 0x65, 0x50, 0x6c, 0x61, 0x79, 0x65, 0x72, 0x0a, 0x3b, 0x20,
        0x68, 0x74, 0x74, 0x70, 0x73, 0x3a, 0x2f, 0x2f, 0x67, 0x69, 0x74, 0x68, 0x75, 0x62, 0x2e, 0x63, 0x6f, 0x6d,
        0x2f, 0x49, 0x73, 0x6f, 0x61, 0x53, 0x46, 0x6c, 0x75, 0x73, 0x2f, 0x51, 0x4c, 0x69, 0x76, 0x65, 0x50, 0x6c,
        0x61, 0x79, 0x65, 0x72, 0x0a, 0x54, 0x69, 0x74, 0x6c, 0x65, 0x3a, 0x20, 0x44, 0x61, 0x6e, 0x6d, 0x61, 0x6b,
        0x75, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a, 0x53, 0x63, 0x72, 0x69, 0x70, 0x74, 0x54, 0x79, 0x70, 0x65, 0x3a,
        0x20, 0x76, 0x34, 0x2e, 0x30, 0x30, 0x2b, 0x0a, 0x57, 0x72, 0x61, 0x70, 0x53, 0x74, 0x79, 0x6c, 0x65, 0x3a,
        0x20, 0x30, 0x0a, 0x53, 0x63, 0x61, 0x6c, 0x65, 0x64, 0x42, 0x6f, 0x72, 0x64, 0x65, 0x72, 0x41, 0x6e, 0x64,
        0x53, 0x68, 0x61, 0x64, 0x6f, 0x77, 0x3a, 0x20, 0x79, 0x65, 0x73, 0x0a, 0x59, 0x43, 0x62, 0x43, 0x72, 0x20,
        0x4d, 0x61, 0x74, 0x72, 0x69, 0x78, 0x3a, 0x20, 0x4e, 0x6f, 0x6e, 0x65, 0x0a, 0x50, 0x6c, 0x61, 0x79, 0x52,
        0x65, 0x73, 0x58, 0x3a, 0x20, 0x31, 0x39, 0x32, 0x30, 0x0a, 0x50, 0x6c, 0x61, 0x79, 0x52, 0x65, 0x73, 0x59,
        0x3a, 0x20, 0x31, 0x30, 0x38, 0x30, 0x0a, 0x0a, 0x5b, 0x56, 0x34, 0x2b, 0x20, 0x53, 0x74, 0x79, 0x6c, 0x65,
        0x73, 0x5d, 0x0a, 0x46, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x3a, 0x20, 0x4e, 0x61, 0x6d, 0x65, 0x2c, 0x20, 0x46,
        0x6f, 0x6e, 0x74, 0x6e, 0x61, 0x6d, 0x65, 0x2c, 0x20, 0x46, 0x6f, 0x6e, 0x74, 0x73, 0x69, 0x7a, 0x65, 0x2c,
        0x20, 0x50, 0x72, 0x69, 0x6d, 0x61, 0x72, 0x79, 0x43, 0x6f, 0x6c, 0x6f, 0x75, 0x72, 0x2c, 0x20, 0x53, 0x65,
        0x63, 0x6f, 0x6e, 0x64, 0x61, 0x72, 0x79, 0x43, 0x6f, 0x6c, 0x6f, 0x75, 0x72, 0x2c, 0x20, 0x4f, 0x75, 0x74,
        0x6c, 0x69, 0x6e, 0x65, 0x43, 0x6f, 0x6c, 0x6f, 0x75, 0x72, 0x2c, 0x20, 0x42, 0x61, 0x63, 0x6b, 0x43, 0x6f,
        0x6c, 0x6f, 0x75, 0x72, 0x2c, 0x20, 0x42, 0x6f, 0x6c, 0x64, 0x2c, 0x20, 0x49, 0x74, 0x61, 0x6c, 0x69, 0x63,
        0x2c, 0x20, 0x55, 0x6e, 0x64, 0x65, 0x72, 0x6c, 0x69, 0x6e, 0x65, 0x2c, 0x20, 0x53, 0x74, 0x72, 0x69, 0x6b,
        0x65, 0x4f, 0x75, 0x74, 0x2c, 0x20, 0x53, 0x63, 0x61, 0x6c, 0x65, 0x58, 0x2c, 0x20, 0x53, 0x63, 0x61, 0x6c,
        0x65, 0x59, 0x2c, 0x20, 0x53, 0x70, 0x61, 0x63, 0x69, 0x6e, 0x67, 0x2c, 0x20, 0x41, 0x6e, 0x67, 0x6c, 0x65,
        0x2c, 0x20, 0x42, 0x6f, 0x72, 0x64, 0x65, 0x72, 0x53, 0x74, 0x79, 0x6c, 0x65, 0x2c, 0x20, 0x4f, 0x75, 0x74,
        0x6c, 0x69, 0x6e, 0x65, 0x2c, 0x20, 0x53, 0x68, 0x61, 0x64, 0x6f, 0x77, 0x2c, 0x20, 0x41, 0x6c, 0x69, 0x67,
        0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x2c, 0x20, 0x4d, 0x61, 0x72, 0x67, 0x69, 0x6e, 0x4c, 0x2c, 0x20, 0x4d, 0x61,
        0x72, 0x67, 0x69, 0x6e, 0x52, 0x2c, 0x20, 0x4d, 0x61, 0x72, 0x67, 0x69, 0x6e, 0x56, 0x2c, 0x20, 0x45, 0x6e,
        0x63, 0x6f, 0x64, 0x69, 0x6e, 0x67, 0x0a, 0x53, 0x74, 0x79, 0x6c, 0x65, 0x3a, 0x20, 0x44, 0x65, 0x66, 0x61,
        0x75, 0x6c, 0x74, 0x2c, 0x53, 0x61, 0x6e, 0x73, 0x2c, 0x34, 0x30, 0x2c, 0x26, 0x48, 0x30, 0x30, 0x46, 0x46,
        0x46, 0x46, 0x46, 0x46, 0x2c, 0x26, 0x48, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x46, 0x46, 0x2c, 0x26, 0x48,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2c, 0x26, 0x48, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x2c, 0x30, 0x2c, 0x30, 0x2c, 0x30, 0x2c, 0x30, 0x2c, 0x31, 0x30, 0x30, 0x2c, 0x31, 0x30, 0x30, 0x2c,
        0x30, 0x2c, 0x30, 0x2c, 0x31, 0x2c, 0x31, 0x2c, 0x30, 0x2c, 0x37, 0x2c, 0x30, 0x2c, 0x30, 0x2c, 0x30, 0x2c,
        0x31, 0x0a, 0x0a, 0x5b, 0x45, 0x76, 0x65, 0x6e, 0x74, 0x73, 0x5d, 0x0a, 0x46, 0x6f, 0x72, 0x6d, 0x61, 0x74,
        0x3a, 0x20, 0x4c, 0x61, 0x79, 0x65, 0x72, 0x2c, 0x20, 0x53, 0x74, 0x61, 0x72, 0x74, 0x2c, 0x20, 0x45, 0x6e,
        0x64, 0x2c, 0x20, 0x53, 0x74, 0x79, 0x6c, 0x65, 0x2c, 0x20, 0x4e, 0x61, 0x6d, 0x65, 0x2c, 0x20, 0x4d, 0x61,
        0x72, 0x67, 0x69, 0x6e, 0x4c, 0x2c, 0x20, 0x4d, 0x61, 0x72, 0x67, 0x69, 0x6e, 0x52, 0x2c, 0x20, 0x4d, 0x61,
        0x72, 0x67, 0x69, 0x6e, 0x56, 0x2c, 0x20, 0x45, 0x66, 0x66, 0x65, 0x63, 0x74, 0x2c, 0x20, 0x54, 0x65, 0x78,
        0x74, 0x0a, 0x12, 0x54, 0xc3, 0x67, 0x40, 0x82, 0xbf, 0x84, 0xc0, 0xe7, 0x7d, 0x7e, 0x73, 0x73, 0x01, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x2e, 0x63, 0xc0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67, 0xc8,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1a, 0x45, 0xa3, 0x87, 0x45, 0x4e, 0x43, 0x4f, 0x44, 0x45, 0x52,
        0x44, 0x87, 0x8d, 0x4c, 0x61, 0x76, 0x66, 0x35, 0x38, 0x2e, 0x32, 0x39, 0x2e, 0x31, 0x30, 0x30, 0x73, 0x73,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3a, 0x63, 0xc0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
        0x63, 0xc5, 0x81, 0x01, 0x67, 0xc8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x45, 0xa3, 0x88, 0x44,
        0x55, 0x52, 0x41, 0x54, 0x49, 0x4f, 0x4e, 0x44, 0x87, 0x94, 0x30, 0x30, 0x3a, 0x30, 0x30, 0x3a, 0x30, 0x35,
        0x2e, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x00, 0x00,
    ]
}

#[derive(Serialize, Debug)]
pub struct DMKVCluster {
    cluster_id: u32,
    cluster_size: u64,
    timestamp_id: u8,
    timestamp_size: u8,
    timestamp: u64,
    block_group_id: u8,
    block_group_size: u32,
    block_id: u8,
    block_size: u32,
    block_content_header: u32,
    block_content: Vec<u8>,
    block_duration_id: u8,
    block_duration_size: u8,
    block_duration_content: u32,
}

impl DMKVCluster {
    pub fn new(ass: Vec<u8>, ts: usize, speed: usize) -> Self {
        let ass_len = ass.len();
        Self {
            cluster_id: 0x1f43b675,
            cluster_size: (ass_len + 30) as u64 | 0x0100_0000_0000_0000,
            timestamp_id: 0xe7,
            timestamp_size: 0x88,
            timestamp: ts as u64,
            block_group_id: 0xa0,
            block_group_size: (ass_len + 15) as u32 | 0x1000_0000u32,
            block_id: 0xa1,
            block_size: (ass_len + 4) as u32 | 0x1000_0000u32,
            block_content_header: 0x8100_0000u32,
            block_content: ass,
            block_duration_id: 0x9b,
            block_duration_size: 0x84,
            block_duration_content: speed as u32,
        }
    }
    pub async fn write_to_socket(&self, socket: &mut Box<dyn DMLStream>) -> Result<()> {
        socket.write_u32(self.cluster_id).await?;
        socket.write_u64(self.cluster_size).await?;
        socket.write_u8(self.timestamp_id).await?;
        socket.write_u8(self.timestamp_size).await?;
        socket.write_u64(self.timestamp).await?;
        socket.write_u8(self.block_group_id).await?;
        socket.write_u32(self.block_group_size).await?;
        socket.write_u8(self.block_id).await?;
        socket.write_u32(self.block_size).await?;
        socket.write_u32(self.block_content_header).await?;
        socket.write(&self.block_content).await?;
        socket.write_u8(self.block_duration_id).await?;
        socket.write_u8(self.block_duration_size).await?;
        socket.write_u32(self.block_duration_content).await?;
        Ok(())
    }
}
