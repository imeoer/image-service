// Copyright 2023 Nydus Developers. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use std::{
    os::{fd::RawFd, unix::net::UnixStream},
    path::PathBuf,
};

use sendfd::{RecvWithFd, SendWithFd};

use super::{Result, StorageBackend, StorageBackendErr};

pub struct UdsStorageBackend {
    uds_path: PathBuf,
}

impl UdsStorageBackend {
    pub fn new(uds_path: PathBuf) -> Self {
        UdsStorageBackend { uds_path }
    }
}

impl StorageBackend for UdsStorageBackend {
    fn save(&mut self, fds: &[RawFd], opaque: &[u8]) -> Result<usize> {
        if fds.is_empty() {
            return Err(StorageBackendErr::NoEnoughFds);
        }

        let stream =
            UnixStream::connect(&self.uds_path).map_err(|err| StorageBackendErr::SendFd(err))?;

        let mut sent = 0;
        let mut fd_sent = false;
        while sent < opaque.len() {
            let _opaque = &opaque[sent..];
            let _fds = if !fd_sent { &fds } else { &[] as &[RawFd] };
            match stream.send_with_fd(_opaque, &_fds) {
                Ok(size) => {
                    sent += size;
                    fd_sent = true;
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::Interrupted
                        || err.kind() == std::io::ErrorKind::WouldBlock
                    {
                        continue;
                    }
                    return Err(StorageBackendErr::SendFd(err));
                }
            }
        }

        Ok(sent)
    }

    fn restore(&mut self) -> Result<(Vec<RawFd>, Vec<u8>)> {
        let stream =
            UnixStream::connect(&self.uds_path).map_err(|err| StorageBackendErr::SendFd(err))?;

        let mut opaque = Vec::new();
        let mut fds = Vec::new();

        loop {
            let mut _opaque: Vec<u8> = vec![0u8; 256 << 10];
            let mut _fds: Vec<RawFd> = vec![0; 8];
            match stream.recv_with_fd(&mut _opaque, &mut _fds) {
                Ok((size, count)) => {
                    if size == 0 && count == 0 {
                        break;
                    }
                    if size != 0 {
                        _opaque.truncate(size);
                        opaque.append(&mut _opaque);
                    }
                    if count != 0 {
                        _fds.truncate(count);
                        fds.append(&mut _fds);
                    }
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::Interrupted
                        || err.kind() == std::io::ErrorKind::WouldBlock
                    {
                        continue;
                    }
                    return Err(StorageBackendErr::RecvFd(err));
                }
            }
        }

        if fds.is_empty() {
            return Err(StorageBackendErr::NoEnoughFds);
        }

        Ok((fds, opaque))
    }
}
