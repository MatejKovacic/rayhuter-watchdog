# Rayhunter watchdog
*Watchdog app for Rayhunter*

Ryhunter sometimes just crashes, so it is useful to have a watchdog process, that will:
- constantly monitor if `rayhunter-daemon` process is running
- **if** `rayhunter-daemon` is not running, it will automatically start Rayhunter
- in that case it will write a notification in `/media/card/crash.log`
- watchdog process is autostarted at startup
- any output of a watchdog process is saved in `/media/card/rayhunter_watchdog.log`
- binary is written in Rust, is compiled for ARM32 architecture and is statically linked

First create a `watcher` directory:
```
cd rayhunter
mkdir watcher
cd watcher
```

## Install `musleabi` toolchain

```
wget https://musl.cc/armv7l-linux-musleabihf-cross.tgz
tar -xzf armv7l-linux-musleabihf-cross.tgz
export PATH="$HOME/rayhunter/watcher/armv7l-linux-musleabihf-cross/bin:$PATH"
```
Now `which armv7l-linux-musleabihf-gcc` should give you:
```
/home/user/rayhunter/watcher/armv7l-linux-musleabihf-cross/bin/armv7l-linux-musleabihf-gcc
```

## Set dependencies and target architecture

```
cargo new rayhunter_watchdog
cd rayhunter_watchdog
nano Cargo.toml 
```

Add this:
```
[dependencies]
libc = "0.2"

[build]
target = "armv7-unknown-linux-musleabihf"
```

```
mkdir .cargo

nano .cargo/config.toml
```
Add this:
```
[build]
target = "armv7-unknown-linux-musleabihf"

[target.armv7-unknown-linux-musleabihf]
linker = "arm-linux-gnueabihf-gcc"
rustflags = ["-C", "target-feature=+crt-static"]
```

## Application
Copy [main.rs](main.rs) from this repository to `src/main.rs`:

Build it:
```
cargo build --release --target=armv7-unknown-linux-musleabihf
```

Check that binary file is statically linked and for the correct architecture:
```
file target/armv7-unknown-linux-musleabihf/release/rayhunter_watchdog
```

Output should be similar to:
```
target/armv7-unknown-linux-musleabihf/release/rayhunter_watchdog: ELF 32-bit LSB executable, ARM, EABI5 version 1 (SYSV), statically linked, BuildID[sha1]=bb62959f25a5cabccc3461e82399688a2d78bf9f, not stripped
```

Copy app to the device:
```
adb push target/armv7-unknown-linux-musleabihf/release/rayhunter_watchdog /media/card/rayhunter_watchdog_daemon
```

Run watchdog app:
```
adb shell
/ # cd /media/card/
./rayhunter_watchdog_daemon
```

## Create startup script

`vi /etc/init.d/rayhunter_watchdog.sh `

Enter this:

```
#! /bin/sh

set -e

case "$1" in
start)
    echo -n "Starting rayhunter watchdog: "
    start-stop-daemon -S -b --make-pidfile --pidfile /tmp/rayhunter_watchdog.pid \
    --startas /bin/sh -- -c "RUST_LOG=info exec /media/card/rayhunter_watchdog_daemon > /media/card/rayhunter_watchdog.log 2>&1"
    echo "done"
    ;;
  stop)
    echo -n "Stopping rayhunter watchdog: "
    start-stop-daemon -K -p /tmp/rayhunter_watchdog.pid
    echo "done"
    ;;
  restart)
    $0 stop
    $0 start
    ;;
  *)
    echo "Usage rayhunter_watchdog { start | stop | restart }" >&2
    exit 1
    ;;
esac

exit 0
```

Make it executable:
```
sudo chmod +x /etc/init.d/rayhunter_watchdog.sh
```

Start the script:
```
/etc/init.d/rayhunter_watchdog.sh start
```

Set autostart:
```
update-rc.d rayhunter_watchdog.sh defaults 99
 Adding system startup for /etc/init.d/rayhunter_watchdog.sh.
```

## Testing

Open terminal and run `adb shell` and check the processes:
```
/ # ps -A | grep ray
17029 root       0:00 ./rayhunter_watchdog_daemon
17135 root       0:01 /media/card/rayhunter-daemon /data/rayhunter/config.toml
17492 root       0:00 grep ray
```
We can see that `rayhunter-daemon` and watchdog apps are running.

Now we kill the running Rayhunter daemon process:
```
/ # killall -9 17135
```

Check the processes and you will see that it is actually not running anymore...
```
/ # ps -A | grep ray
17029 root       0:00 ./rayhunter_watchdog_daemon
17514 root       0:00 grep ray
```

However after a few seconds it is respawned:
```
/ # ps -A | grep ray
17029 root       0:00 ./rayhunter_watchdog_daemon
17531 root       0:00 /media/card/rayhunter-daemon /data/rayhunter/config.toml
17612 root       0:00 grep ray
```

We can also check `crash.log` to see when it crashed (please note that times are local, time zones are not taken into account):
```
/ # cat /media/card/crash.log 
[2025-04-07 09:39:37] Daemon not running. Restarting...
[2025-04-07 09:39:37] Daemon started.
```
