#!/bin/bash
# KolibriOS AI Android Virtual Device Setup Script
# This script sets up an AVD for testing KolibriOS AI components

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
AVD_NAME="kolibrios_ai_avd"
API_LEVEL=34
DEVICE_DEFINITION="pixel_6"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing=()
    
    # Check for Java
    if ! command_exists java; then
        missing+=("java")
    fi
    
    # Check for Android SDK
    if [ -z "$ANDROID_HOME" ]; then
        if [ -d "$HOME/Android/Sdk" ]; then
            export ANDROID_HOME="$HOME/Android/Sdk"
        elif [ -d "$HOME/Library/Android/sdk" ]; then
            export ANDROID_HOME="$HOME/Library/Android/sdk"
        else
            missing+=("android-sdk")
        fi
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing prerequisites: ${missing[*]}"
        log_info "Please install the following:"
        log_info "  - Java JDK 17 or later"
        log_info "  - Android SDK (via Android Studio or command line tools)"
        exit 1
    fi
    
    # Add SDK tools to PATH
    export PATH="$PATH:$ANDROID_HOME/cmdline-tools/latest/bin"
    export PATH="$PATH:$ANDROID_HOME/platform-tools"
    export PATH="$PATH:$ANDROID_HOME/emulator"
    
    log_info "Prerequisites satisfied"
}

# Install Android SDK components
install_sdk_components() {
    log_info "Installing Android SDK components..."
    
    local sdk_manager="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
    
    if [ ! -f "$sdk_manager" ]; then
        log_error "SDK Manager not found at $sdk_manager"
        exit 1
    fi
    
    # Accept licenses
    log_info "Accepting SDK licenses..."
    yes | "$sdk_manager" --licenses >/dev/null 2>&1 || true
    
    # Install required components
    log_info "Installing platform tools, build tools, and emulator..."
    "$sdk_manager" --install \
        "platform-tools" \
        "platforms;android-$API_LEVEL" \
        "build-tools;$API_LEVEL.0.0" \
        "emulator" \
        "system-images;android-$API_LEVEL;google_apis;x86_64" \
        2>&1 | grep -v "^=" || true
    
    log_info "SDK components installed"
}

# Create AVD
create_avd() {
    log_info "Creating Android Virtual Device..."
    
    local avd_manager="$ANDROID_HOME/cmdline-tools/latest/bin/avdmanager"
    
    # Check if AVD already exists
    if [ -d "$HOME/.android/avd/$AVD_NAME.avd" ]; then
        log_warn "AVD '$AVD_NAME' already exists. Deleting..."
        rm -rf "$HOME/.android/avd/$AVD_NAME.avd"
        rm -f "$HOME/.android/avd/$AVD_NAME.ini"
    fi
    
    # Create AVD
    log_info "Creating AVD with API level $API_LEVEL..."
    echo "no" | "$avd_manager" create avd \
        -n "$AVD_NAME" \
        -k "system-images;android-$API_LEVEL;google_apis;x86_64" \
        -d "$DEVICE_DEFINITION" \
        --force
    
    log_info "AVD created successfully"
}

# Configure AVD hardware
configure_avd() {
    log_info "Configuring AVD hardware..."
    
    local config_file="$HOME/.android/avd/$AVD_NAME.avd/config.ini"
    
    if [ ! -f "$config_file" ]; then
        log_error "AVD config file not found: $config_file"
        exit 1
    fi
    
    # Add hardware configuration
    cat >> "$config_file" << EOF

# KolibriOS AI Custom Configuration
hw.ramSize=4096
hw.cpu.ncore=4
hw.gpu.enabled=yes
hw.gpu.mode=host
hw.lcd.width=1080
hw.lcd.height=2340
hw.lcd.density=420
hw.keyboard=yes
hw.sensors.orientation=yes
hw.sensors.accelerometer=yes
hw.sensors.gyroscope=yes
hw.sensors.proximity=yes
vm.heapSize=512
disk.dataPartition.size=4G
fastboot.forceColdBoot=no
EOF
    
    log_info "AVD configured"
}

# Create launch script
create_launch_script() {
    log_info "Creating launch script..."
    
    local launch_script="$PROJECT_ROOT/scripts/launch_kolibrios_android_avd.sh"
    
    cat > "$launch_script" << 'SCRIPT'
#!/bin/bash
# Launch KolibriOS AI Android AVD

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
AVD_NAME="kolibrios_ai_avd"

# Set Android SDK path
if [ -z "$ANDROID_HOME" ]; then
    if [ -d "$HOME/Android/Sdk" ]; then
        export ANDROID_HOME="$HOME/Android/Sdk"
    elif [ -d "$HOME/Library/Android/sdk" ]; then
        export ANDROID_HOME="$HOME/Library/Android/sdk"
    else
        echo "ERROR: ANDROID_HOME not set"
        exit 1
    fi
fi

EMULATOR="$ANDROID_HOME/emulator/emulator"

if [ ! -f "$EMULATOR" ]; then
    echo "ERROR: Emulator not found at $EMULATOR"
    exit 1
fi

echo "Starting KolibriOS AI AVD..."
echo "AVD Name: $AVD_NAME"
echo ""

# Launch emulator with options
"$EMULATOR" -avd "$AVD_NAME" \
    -memory 4096 \
    -cores 4 \
    -gpu host \
    -no-audio \
    -no-boot-anim \
    -accel on \
    -netdelay none \
    -netspeed full \
    "$@"
SCRIPT
    
    chmod +x "$launch_script"
    log_info "Launch script created: $launch_script"
}

# Create KolibriOS AI Android runtime project structure
create_android_runtime() {
    log_info "Creating Android runtime project structure..."
    
    local runtime_dir="$PROJECT_ROOT/android_runtime"
    
    mkdir -p "$runtime_dir/app/src/main/java/com/kolibrios/ai"
    mkdir -p "$runtime_dir/app/src/main/res/layout"
    mkdir -p "$runtime_dir/app/src/main/res/values"
    mkdir -p "$runtime_dir/app/src/main/assets"
    
    # Create build.gradle
    cat > "$runtime_dir/app/build.gradle" << 'GRADLE'
plugins {
    id 'com.android.application'
}

android {
    namespace 'com.kolibrios.ai'
    compileSdk 34

    defaultConfig {
        applicationId "com.kolibrios.ai"
        minSdk 24
        targetSdk 34
        versionCode 1
        versionName "1.0"
    }

    buildTypes {
        release {
            minifyEnabled false
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
        }
    }

    compileOptions {
        sourceCompatibility JavaVersion.VERSION_17
        targetCompatibility JavaVersion.VERSION_17
    }
}

dependencies {
    implementation 'androidx.appcompat:appcompat:1.6.1'
    implementation 'com.google.android.material:material:1.11.0'
    implementation 'androidx.constraintlayout:constraintlayout:2.1.4'
}
GRADLE

    # Create AndroidManifest.xml
    cat > "$runtime_dir/app/src/main/AndroidManifest.xml" << 'MANIFEST'
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    
    <application
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:roundIcon="@mipmap/ic_launcher_round"
        android:supportsRtl="true"
        android:theme="@style/Theme.KolibriOSAI">
        
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:label="@string/app_name"
            android:theme="@style/Theme.KolibriOSAI">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
        
    </application>
</manifest>
MANIFEST

    # Create MainActivity.java
    cat > "$runtime_dir/app/src/main/java/com/kolibrios/ai/MainActivity.java" << 'JAVA'
package com.kolibrios.ai;

import android.os.Bundle;
import androidx.appcompat.app.AppCompatActivity;

/**
 * KolibriOS AI Android Runtime - Main Activity
 * 
 * This activity serves as the entry point for the KolibriOS AI
 * Android runtime environment. It hosts the GUI and Living Applications.
 */
public class MainActivity extends AppCompatActivity {
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        // Initialize KolibriOS AI runtime
        initializeRuntime();
    }
    
    private void initializeRuntime() {
        // TODO: Initialize KolibriOS AI components
        // - Load native libraries
        // - Start communication with kernel/cells
        // - Set up GUI framework
    }
}
JAVA

    # Create layout
    cat > "$runtime_dir/app/src/main/res/layout/activity_main.xml" << 'LAYOUT'
<?xml version="1.0" encoding="utf-8"?>
<androidx.constraintlayout.widget.ConstraintLayout
    xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:app="http://schemas.android.com/apk/res-auto"
    android:layout_width="match_parent"
    android:layout_height="match_parent">
    
    <TextView
        android:id="@+id/titleText"
        android:layout_width="wrap_content"
        android:layout_height="wrap_content"
        android:text="KolibriOS AI"
        android:textSize="32sp"
        android:textStyle="bold"
        app:layout_constraintCenter_horizontal="true"
        app:layout_constraintTop_toTopOf="parent"
        android:layout_marginTop="100dp"/>
    
    <TextView
        android:id="@+id/statusText"
        android:layout_width="wrap_content"
        android:layout_height="wrap_content"
        android:text="Runtime Initializing..."
        android:textSize="16sp"
        app:layout_constraintCenter_horizontal="true"
        app:layout_constraintTop_toBottomOf="@id/titleText"
        android:layout_marginTop="20dp"/>
    
    <ProgressBar
        android:id="@+id/loadingIndicator"
        android:layout_width="wrap_content"
        android:layout_height="wrap_content"
        app:layout_constraintCenter_horizontal="true"
        app:layout_constraintTop_toBottomOf="@id/statusText"
        android:layout_marginTop="20dp"/>

</androidx.constraintlayout.widget.ConstraintLayout>
LAYOUT

    # Create strings.xml
    cat > "$runtime_dir/app/src/main/res/values/strings.xml" << 'STRINGS'
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">KolibriOS AI</string>
    <string name="runtime_initializing">Runtime Initializing...</string>
    <string name="runtime_ready">Runtime Ready</string>
    <string name="connection_error">Connection Error</string>
</resources>
STRINGS

    # Create README
    cat > "$runtime_dir/README.md" << 'README'
# KolibriOS AI Android Runtime

This is the Android runtime environment for KolibriOS AI.

## Building

1. Open the project in Android Studio
2. Sync Gradle files
3. Build -> Make Project

## Running

1. Launch the AVD: `./scripts/launch_kolibrios_android_avd.sh`
2. Run the app from Android Studio or: `./gradlew installDebug`

## Architecture

The Android runtime provides:
- Native library loading (JNI)
- Communication bridge to KolibriOS AI kernel
- GUI framework for Living Applications
- Sensor and system integration
README

    log_info "Android runtime project created: $runtime_dir"
}

# Main setup process
main() {
    log_info "============================================"
    log_info "KolibriOS AI Android Virtual Device Setup"
    log_info "============================================"
    echo ""
    
    check_prerequisites
    install_sdk_components
    create_avd
    configure_avd
    create_launch_script
    create_android_runtime
    
    echo ""
    log_info "============================================"
    log_info "Setup Complete!"
    log_info "============================================"
    echo ""
    log_info "To launch the AVD, run:"
    log_info "  ./scripts/launch_kolibrios_android_avd.sh"
    echo ""
    log_info "To build and install the Android runtime:"
    log_info "  cd android_runtime"
    log_info "  ./gradlew installDebug"
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --api-level)
            API_LEVEL="$2"
            shift 2
            ;;
        --device)
            DEVICE_DEFINITION="$2"
            shift 2
            ;;
        --skip-sdk)
            SKIP_SDK=true
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --api-level LEVEL   Android API level (default: 34)"
            echo "  --device DEVICE     Device definition (default: pixel_6)"
            echo "  --skip-sdk          Skip SDK component installation"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

main "$@"
