# Rhythm Mode Music Tracks

Music tracks from Crypt of the NecroDancer OST by Danny Baranowsky.

## Current Tracks
- `track1.mp3` - Watch Your Step (Training) - 120 BPM
- `track2.mp3` - Crypteque (1-2) - 130 BPM
- `track3.mp3` - Tombtorial (Tutorial) - 100 BPM
- `track4.mp3` - Disco Descent (1-1) - 115 BPM
- `track5.mp3` - Mausoleum Mash (1-3) - 140 BPM

## How It Works
- A random track is selected when rhythm mode game starts
- The same track loops throughout the game session
- BPM is synced to the selected track automatically

## Adding New Tracks
1. Add MP3 file as `trackN.mp3`
2. Update `MUSIC_TRACKS` array in `src/lib/game/rhythmEngine.ts`
3. Include: name, url, bpm, beatOffset (usually 0)

## Requirements
- MP3 format recommended
- Must have a clear, consistent beat
- The BPM must match what's configured in `rhythmEngine.ts`

## Tips
- Choose music with a strong, clear beat
- Electronic/chiptune music works well for rhythm games
- Avoid music with tempo changes
- Test the music to ensure the beat aligns properly
