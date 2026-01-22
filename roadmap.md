# Feature Roadmap

## Suggested Future Features

### Tagging & Organization System
- **Custom Tag Creation**: Allow users to create, edit, and delete custom tags (e.g., "Work", "Photos", "Archive")
- **Bulk Tagging**: Select multiple files and apply tags in one action
- **Tag-Based Filtering**: Filter file views by one or multiple tags
- **Auto-Tagging Rules**: Create rules to automatically tag files based on extension, folder, or name patterns
- **Tag Colors & Icons**: Visual customization for easier identification

### Database & Persistence
- **Local SQLite Database**: Lightweight embedded database for storing file metadata and tags
- **Database Export/Import**: Backup and restore tag data
- **File Index Caching**: Store hash results to speed up repeat scans
- **Database Search**: Fast query-based file lookup by name, tag, size, or date
- **Multiple Database Profiles**: Support different databases for different projects/folders

### Advanced Duplicate Detection
- **Fuzzy Name Matching**: Detect similar filenames (e.g., "photo1.jpg" vs "photo_1.jpg")
- **Extention Detection**: Detect similar filenames with different extensions (e.g., "photo1.jpg" vs "photo1.png")
- **Image Similarity Detection**: Find visually similar images using perceptual hashing
- **Audio Fingerprinting**: Detect duplicate audio files even with different metadata
- **Content Preview**: Preview files before deciding to delete/move
- **Duplicate Age Priority**: Suggest keeping the oldest or newest file

### Sorting & File Management
- **Custom Sort Rules**: Sort files into folders based on extension, date, size, or tags
- **Scheduled Scans**: Automatic periodic scanning of designated folders
- **Folder Watching**: Real-time monitoring for new duplicates
- **Batch Rename**: Rename files using patterns and templates
- **File Compression**: Option to compress duplicates instead of deleting

### User Experience Enhancements
- **Dark/Light Theme Toggle**: User-selectable color schemes
- **Scan History**: Log of previous scans with results summary
- **Undo Support**: Recover recently deleted/moved files
- **Keyboard Shortcuts**: Power-user navigation
- **Drag & Drop**: Drop folders onto the app to scan
- **Context Menu Integration**: Right-click "Scan for duplicates" in Windows Explorer
- **System Tray Mode**: Minimize to tray for background monitoring

### Reporting & Export
- **HTML/PDF Reports**: Generate shareable duplicate reports
- **CSV Export**: Export file lists for external processing
- **Statistics Dashboard**: Visual charts showing storage usage, duplicate counts, etc.
- **Space Savings Calculator**: Show potential disk space recovery

### Performance & Scale
- **Network Drive Support**: Scan mapped drives and UNC paths
- **Multi-Folder Scanning**: Scan multiple directories in one session
- **Exclusion Patterns**: Ignore specific folders, file types, or name patterns
- **Pause/Resume Scanning**: Interrupt long scans without losing progress
- **Memory-Efficient Mode**: Stream processing for very large directories

### Integration & Extensibility
- **Plugin System**: Allow third-party extensions
- **Command-Line Interface**: CLI mode for scripting and automation
- **Cloud Storage Support**: Scan OneDrive, Google Drive, Dropbox folders
- **API/Scripting**: Expose functionality for external tools
