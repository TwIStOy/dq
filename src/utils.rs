use std::{env, ffi::CString, os::fd::FromRawFd};

use anyhow::bail;
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::Response;
use tokio::io::AsyncWriteExt;

use crate::config::Config;

