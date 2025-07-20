# sRow motion

A Rust application that safely moves all files and directories from a source directory to a destination directory with data integrity guarantees.


## Overview

sRow motion is a command-line tool designed to move all contents from a specified source directory to a destination directory. The application is designed to be used with system schedulers such as cron or Task Scheduler.  
The application includes following features:

- **Scheduled execution**: Only runs on specified weekdays
- **Data integrity verification**: Ensures no data loss during transfer
- **Rollback capability**: Automatically reverts changes if transfer fails
- **Path template support**: Support for dynamic destination paths using `{yyyy}`, `{mm}`, `{dd}` placeholders

## Architecture

The project follows Clean Architecture principles with the following structure:

```
sRow-motion/
├── adapter/          # Interface adapters that mediate between domain and infra
├── domain/           # Business logic and core entities
├── infra/            # Infrastructure concerns (file system operations)
├── shared/           # Shared utilities and error types
└── src/              # Application entry point
```

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- **Note**: Currently tested only on PowerShell (Windows). Other shells and operating systems may require additional testing.

### Building from source

```bash
git clone <repository-url>
cd sRow-motion
cargo build --release
```

The binary will be available at `target/release/srow`.

## Usage

### Configuration File Method

Create a JSON configuration file:

```json
{
    "source_directory_path": "C:\\Users\\hoge\\Desktop\\",
    "destination_directory_path": "C:\\Users\\hoge\\Files\\{yyyy}\\{mm}\\{dd}\\",
    "weekday": "Thu"
}
```

Run the application:

```powershell
srow --file config.json
```

### Command Line Arguments Method

```powershell
srow `
    --source-directory "C:\Users\hoge\Desktop\" `
    --destination-directory "C:\Users\hoge\Files\{yyyy}\{mm}\{dd}\" `
    --weekday "Thu"
```

### Configuration Parameters

- **source_directory_path**: Source directory containing files to move (absolute path required)
- **destination_directory_path**: Target directory for moved files (absolute path required)
  - Supports placeholders: `{yyyy}` (4-digit year), `{mm}` (2-digit month), `{dd}` (2-digit day)
- **weekday**: Day of the week to execute the transfer
  - Valid values: "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"



## Safety Features

### Data Integrity

1. **Copy-then-verify**: Files are copied to destination first
2. **Content verification**: Source and destination contents are compared
3. **Automatic rollback**: If verification fails, destination is cleaned up
4. **Source removal**: Source files are only removed after successful verification

## Development

### Running Tests

```bash
cargo test
```
## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.