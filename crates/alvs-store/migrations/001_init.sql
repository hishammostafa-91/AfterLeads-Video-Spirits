-- SPDX-License-Identifier: AGPL-3.0-or-later
-- Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
-- Initial cache schema, verbatim from docs/03 §3.7 (WP-03 card, T002).
-- Runs inside one transaction by the alvs-store migration runner; the
-- schema version is recorded in meta.schema_version.

CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);

CREATE TABLE media_cache (
  media_key   TEXT PRIMARY KEY,           -- SHA-256 of the canonical fingerprint JSON
  path        TEXT NOT NULL,
  probe_json  TEXT NOT NULL,
  probed_utc  TEXT NOT NULL
);

CREATE TABLE thumbnails (
  media_key     TEXT NOT NULL,
  kind          TEXT NOT NULL CHECK (kind IN ('poster', 'filmstrip')),
  height        INTEGER NOT NULL,
  interval_flicks INTEGER NOT NULL,       -- 0 for poster
  file          TEXT NOT NULL,            -- relative to the cache folder
  created_utc   TEXT NOT NULL,
  last_used_utc TEXT NOT NULL,
  PRIMARY KEY (media_key, kind, height)
);

CREATE TABLE waveforms (
  media_key     TEXT NOT NULL,
  stream        INTEGER NOT NULL,
  file          TEXT NOT NULL,
  created_utc   TEXT NOT NULL,
  last_used_utc TEXT NOT NULL,
  PRIMARY KEY (media_key, stream)
);

CREATE TABLE proxies (
  media_key     TEXT PRIMARY KEY,
  profile       TEXT NOT NULL,            -- e.g. 'h264-intra-720'
  file          TEXT,
  state         TEXT NOT NULL CHECK (state IN ('queued', 'building', 'ready', 'failed')),
  error_code    TEXT,
  created_utc   TEXT NOT NULL,
  last_used_utc TEXT NOT NULL
);

CREATE TABLE jobs (
  id           TEXT PRIMARY KEY,
  kind         TEXT NOT NULL,
  state        TEXT NOT NULL CHECK (state IN ('queued','running','paused','succeeded','failed','cancelled')),
  priority     INTEGER NOT NULL,
  project_id   TEXT,
  spec_json    TEXT NOT NULL,
  progress     REAL NOT NULL DEFAULT 0,
  attempts     INTEGER NOT NULL DEFAULT 0,
  error_code   TEXT,
  error_detail TEXT,
  output_json  TEXT,
  log_file     TEXT,
  created_utc  TEXT NOT NULL,
  started_utc  TEXT,
  finished_utc TEXT
);
CREATE INDEX jobs_by_state ON jobs (state, priority DESC, created_utc);

CREATE TABLE recent_projects (
  path          TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  thumbnail_file TEXT,
  opened_utc    TEXT NOT NULL,
  pinned        INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE models (
  id            TEXT PRIMARY KEY,           -- e.g. 'small-q5_1'
  file          TEXT NOT NULL,
  sha256        TEXT NOT NULL,
  size          INTEGER NOT NULL,
  state         TEXT NOT NULL CHECK (state IN ('downloading', 'installed', 'corrupt')),
  installed_utc TEXT
);

CREATE TABLE encoder_probe (
  driver_key  TEXT PRIMARY KEY,             -- adapter vendor/device + driver version
  result_json TEXT NOT NULL,
  probed_utc  TEXT NOT NULL
);

CREATE TABLE analysis (
  media_key   TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('loudness', 'stabilize', 'transcript')),
  params_hash TEXT NOT NULL,
  file        TEXT NOT NULL,
  created_utc TEXT NOT NULL,
  PRIMARY KEY (media_key, kind, params_hash)
);
