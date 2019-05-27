# Changelog

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 27.05.2019

### Added
- Option for source string added to torrent info (-s), included in infohash.
Often used by private trackers to create a unique infohash to prevent peer-leak
and the possibility to track the trackers that do use leaked torrents.
Having this option in maketorrent make it possible to create a infohash accurate
torrent to the tracker you want to upload it to

## [0.1.1] - 05.05.2018

### Fixed
- Use a unreleased version of `bip_metainfo` to fix the setter bugs

### Changed
- Non Verbose mode now shows a progress text of the hashed pieces.

### Added
- Multiple Announce URLs are now supported

## [0.1.0]
- Initial Version
