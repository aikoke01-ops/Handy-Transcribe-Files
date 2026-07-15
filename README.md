# Handy 0.9.2-transcribe

**Extended version of Handy with file transcription support**

A free, open-source, and extensible speech-to-text application that works completely offline. This fork adds the ability to transcribe audio and video files from your computer, not just live microphone input.

## ✨ What's New (v0.9.2-transcribe)

### File Transcription Feature
- **Transcribe audio files** — MP3, WAV, FLAC, OGG, M4A, AAC, and more
- **Transcribe video files** — MP4, MKV, MOV, AVI, WebM (with ffmpeg)
- **No internet required** — Everything runs locally on your machine
- **Privacy-first** — Your audio never leaves your computer
- **Better error handling** — Clear error messages instead of app crashes
- **Easy-to-use interface** — Simple file picker and one-click transcription

### Supported Audio Formats (Built-in)
- ✅ MP3
- ✅ WAV
- ✅ FLAC
- ✅ OGG / Vorbis
- ✅ M4A / AAC
- ✅ PCM

### Video/Advanced Formats (With ffmpeg)
- MP4, MKV, MOV, AVI, WebM, and more

## 🚀 Installation

### Windows (Recommended)
1. Download `Handy_0.9.2-transcribe_Installer.exe`
2. Run the installer
3. Follow the setup wizard
4. Launch from Start Menu or Desktop shortcut

### Manual/Portable (Windows)
1. Extract the release ZIP to a folder
2. Run `handy.exe` directly

### macOS / Linux
Build from source (see below)

## 💻 How to Use

### Live Microphone Recording (Original Feature)
1. Press your configured shortcut key
2. Speak into your microphone
3. Release to transcribe

### Transcribe Files (New Feature)
1. Open Handy
2. Go to **"Transcribe File"** in the left menu
3. Click **"Choose file..."** and select your audio/video file
4. Click **"Transcribe"**
5. Wait for processing (time depends on file length)
6. Result appears below — click the copy icon to copy to clipboard

## ⚙️ Requirements

### Minimum
- Windows 10/11 or macOS/Linux
- 4GB RAM recommended (8GB+ for larger files)
- Speech-to-text model (auto-downloads on first use)

### Optional
- **ffmpeg** — For transcribing MP4, MKV, AVI, and other video formats
  - Download from: https://ffmpeg.org/download.html
  - Add to system PATH for automatic support

### Model Selection
- **Small** — Fastest, lowest quality, ~440 MB
- **Medium** — Balanced, ~1.5 GB
- **Large** — Best quality, slowest, ~3 GB

**Tip:** Start with "Small" if you have limited RAM.

## 🔧 Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) 1.70+
- [Node.js](https://nodejs.org/) 18+
- [Bun](https://bun.sh/) (recommended) or npm
- CMake 3.20+
- [Visual Studio Community](https://visualstudio.microsoft.com/) (Windows)

### Steps

```bash
# Clone the repository
git clone https://github.com/aikoke01-ops/Handy-Transcribe-Files.git
cd Handy

# Install dependencies
bun install

# Compile and run in development mode
bun run tauri dev

# Build release version
bun run tauri build
```

## 📋 Key Changes from Original

| Feature | Original Handy | This Fork |
|---------|---|---|
| Live Mic Recording | ✅ | ✅ |
| File Transcription | ❌ | ✅ NEW |
| Error Messages | Basic | Enhanced |
| Supported Audio Formats | Limited | Extensive |
| GUI for File Selection | ❌ | ✅ NEW |

## 🐛 Known Issues

1. **Large files (>1 hour)** — May use significant RAM. Consider transcribing in chunks.
2. **Video files without audio** — Will fail with "No audio detected" error.
3. **ffmpeg required for some codecs** — MP4 files with certain audio codecs need ffmpeg installed.

## 🙏 Credits

- **Original Handy** — [cjpais/Handy](https://github.com/cjpais/Handy)
- **File Transcription Feature** — This fork (2024)
- **Whisper Models** — OpenAI
- **Symphonia Decoder** — Pure-Rust audio decoding
- **FFmpeg Fallback** — For advanced video/audio format support

## 📄 License

MIT License — Same as original Handy

## 🤝 Contributing

Found a bug? Have a feature idea?

1. **For issues with file transcription** — Open an issue here
2. **For general Handy issues** — See the [original repository](https://github.com/cjpais/Handy)

## 📞 Support

### File Transcription Not Working?

**MP3/WAV not transcribing?**
- Make sure a model is selected (go to Settings → Models)
- Try with "Small" model first
- Check that the file isn't corrupted

**MP4/MKV files fail to decode?**
- Install ffmpeg from https://ffmpeg.org/download.html
- Add to system PATH
- Restart Handy

**Out of memory error?**
- Close other applications
- Switch to "Small" or "Turbo" model
- Transcribe shorter files

**Still need help?**
- Check error messages carefully — they now describe exactly what went wrong
- Try the original Handy: https://github.com/cjpais/Handy

## 🚀 Roadmap

Potential future improvements:
- [ ] Batch file transcription
- [ ] Transcription history/logs
- [ ] Custom language selection per file
- [ ] Real-time transcription progress bar
- [ ] Export to different formats (SRT, VTT, etc.)

---

**Enjoy transcribing!** 🎙️➡️📝
