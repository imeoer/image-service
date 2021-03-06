/*
 * Copyright (c) 2020. Ant Group. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package process

import (
	"contrib/nydus-snapshotter/pkg/daemon"
	"contrib/nydus-snapshotter/pkg/store"
)

type Store interface {
	Get(id string) (*daemon.Daemon, error)
	GetBySnapshot(snapshotID string) (*daemon.Daemon, error)
	Add(*daemon.Daemon) error
	Delete(*daemon.Daemon)
	List() []*daemon.Daemon
	Size() int
}

var _ Store = &store.DaemonStore{}
