
# Podchamp
## what
A no-frills tool for fetching your podcasts.

**Work in progress,** but I've been using it exclusively for six months without any major issues.

## why
I've found very few podcast tools that aren't either horribly bloated (I don't want my downloader to
also be a half-assed media player), decent-but-fragile (shell scripts; tries too hard to make
sense of hopelessly broken feeds), or inflexible (insists on performing the download itself; wants
to know filename patterns and directories).

I've used [greg](https://github.com/manolomartinez/greg) for years now, but that project is
more-or-less dead, and I don't like a few of its design choices, so this is a replacement of sorts.

## who
It's for me. This will not get lots of extraneous features like playback or tagging; don't @ me.

## how
Podchamp keeps a record of feeds and previously-downloaded episodes in a small database. When
checking for new episodes, podchamp downloads the feed xml, parses it, and launches downloads for
any episodes it doesn't remember downloading previously.

You add a feed like this:

```sh
$ podchamp add mbmbam 'https://feeds.simplecast.com/wjQvYtdl'
```

and then fetch new episodes like so:

```sh
$ podchamp fetch
Fetching mbmbam
```

If there are new episodes of any of your feeds, podchamp launches `PODCHAMP_DOWNLOADER` for each of
them, with the download link as the sole argument. By default this is just wget, which is not
super-useful; it's intended that you write your own script that does whatever you feel is
appropriate. For example, here's a simplified version of mine:

```fish
#!/usr/bin/fish
set dir "$HOME/podcasts/$PODCHAMP_FEED"
mkdir -p "$dir"
cd "$dir"
wget -q "$argv[1]" -O - | \
    nice ffmpeg -y \
    -i pipe:0 -c:a libopus -b:a 64k \
    -metadata title="$PODCHAMP_DATE - $PODCHAMP_TITLE" \
    -metadata artist="$PODCHAMP_FEED" \
    "$PODCHAMP_DATE - $PODCHAMP_TITLE.opus"
```

As you can see, the downloader is also passed a few useful things in environment variables:
- `PODCHAMP_FEED`: this is the name you gave the feed when you `add`ed it.
- `PODCHAMP_DATE`: the publication date of the episode, `yyyy-mm-dd`.
- `PODCHAMP_TITLE`: the title of the episode.

Normally, when you add a new feed, it has a _backlog_ of 1. This means it will download only the
most recent episode the first time you fetch, and every episode newer than it subsequently. If you
want more to be going on with, you can set a larger backlog:

```sh
$ podchamp add -n 10 streetfight 'http://feeds.feedburner.com/streetfightradio'
```
or, if you added it already (or reset the feed) you can change the backlog:

```sh
$ podchamp mod streetfight backlog 10
```

This will download the 10 most recent episodes the first time you fetch, and every episode newer
than the oldest of those subequently.

Currently there's no way to download particular episodes, but I'll implement it eventually.

If you decide you don't like a podcast and want podchamp to stop fetching it, you can
remove its feed:

```sh
# to hell with bean dad
$ podchamp rm roderickontheline
```

In case you want to re-download previous episodes of a podcast (say, you lost the files, or want
fresh copies to transcode them differently), you can reset the feed:

```sh
$ podchamp reset guaranteedaudio
```

## when

This will be considered done (i.e. 1.0) when I'm happy with it.

## where

On linux, definitely. On windows or other platforms, possibly, but you're on your own; please don't
ask for support.

## todo
### yes

- Config file - pretty minimal, probably.
- More metadata - only a few pieces of feed and episode metadata are available to download scripts,
  and it would be trivial to export more.
- Tests - duh.
- Better docs - the above is incomplete; I need to explain environment variables, command-line
  options, and `fetch-since` dates.

### maybe

- Self-downloading - this could be added without too much hassle or bloat, but really it's
  out-of-scope.
- More robust ordering and episode tracking - currently, podchamp ignores any feed item that has no
  `pubDate` or `guid`; this might be improvable without undue work.
- Feeds listed in a text file - every time I think about this, it seems inherently brittle and
  error-prone; an "import" feature might be useful, though.

### nope

- Playback - is what media players are for.
- Tagging - an absolute tar-pit of complexity; can already be accomplished more reliably by existing
  tools
- Support for broken feeds - we may be stuck with the distaster that is RSS, but I have no interest
  in making podchamp try to work around _totally_ mangled feeds.

## whence

[Rachel Knight](https://automorphi.city/).

## whither

All code here is MIT licensed.

