# AirCard for Windows

AirCard is a native Windows client for preparing Apple Wallet card artwork and passcode-theme assets for a paired iPhone. It uses Apple Mobile Device services, AFC, StreamingZip, and AirTraffic. The project is experimental: Apple does not document these private services, and a successful device scan does not prove that every write operation will work on a particular iOS build.

## Version 1.3.0

- Transfers StreamingZip archives through a testable partial-write loop.
- Uses a 64 KiB standard profile and a 16 KiB compatibility profile after a fresh device and service reconnect.
- Shows transfer mode, byte counts, native return values, and retry context in the log and status area.
- Adds a Diagnostics panel for the Apple runtime, device session, AFC, StreamingZip service, and selected card path.
- Reports Store-iTunes/CoreFP and 32-bit/64-bit runtime warnings without altering installed Apple software or the registry.

## Requirements

- Windows 10 or Windows 11, 64-bit.
- A current 64-bit Apple Mobile Device runtime. The classic iTunes installer or Apple Devices installation is the most predictable option for this application.
- Apple Mobile Device Service running. Bonjour is required for Wi-Fi pairing and discovery.
- An iPhone trusted with this Windows account. Use USB for initial pairing.
- A recent iOS release that still exposes the required Apple services. AirCard detects the device version but does not assume a fixed maximum iOS version.

Microsoft Store iTunes can place `CoreFP` registration in package-private state. That can allow device detection and Wallet scanning while preventing AirTraffic or archive transmission. The Diagnostics panel reports this configuration; it never copies DLLs, edits the registry, or removes Apple software.

## Install and build

Download `aircard.exe` from this fork's releases, or build from source with the stable MSVC Rust toolchain:

```powershell
git clone https://github.com/nexis-cmyk/AirCard-Windows.git
cd AirCard-Windows
cargo test
cargo build --release
```

The executable is written to `target\release\aircard.exe`.

## Diagnostics

Open **Help → Run Diagnostics** after selecting an iPhone. The non-destructive checks report:

- Apple Mobile Device Support and the required DLLs
- Apple Mobile Device Service and Bonjour status
- Microsoft Store Apple software, CoreFP configuration, and mixed 32-bit runtime warnings
- detected iPhone, pairing/session, AFC, and StreamingZip service handshake
- Books directory access and the selected card-hash directory, where applicable

The diagnostics pass deliberately does not transfer a test archive or read it back: even a small StreamingZip test has a device-side write effect. A skipped transfer probe is reported explicitly rather than being presented as a successful test.

## Applying Wallet artwork

1. Connect and unlock the iPhone. Select it in AirCard.
2. In **Wallet**, click **Scan**, open Wallet on the iPhone, and select the target card.
3. Choose an image, position the crop, then click **Apply Card Skin**.
4. Follow the visible transfer state. AirCard tries these modes in order:
   - **Mode A:** atomic batch with the 64 KiB standard StreamingZip profile.
   - **Mode B:** individual assets, each with a fresh device session, using the standard profile.
   - **Mode C:** individual assets with a newly opened session and StreamingZip service, using 16 KiB compatibility chunks.
5. Reopen Wallet after a successful write.

The activity log records timestamps, operation names, bytes attempted and transferred, native send results, chunk size, and retry context. It never records Apple IDs, passwords, or pairing secrets.

## Troubleshooting

`AMDServiceConnectionSend` failing during archive transmission is below AFC and AirTraffic. It means the StreamingZip connection stopped accepting archive bytes. Check Diagnostics before repeatedly retrying:

- Use one consistent 64-bit Apple runtime. A stale 32-bit Mobile Device Support folder can be selected accidentally by older tools.
- Avoid mixing 3uTools/i4Tools drivers with the Apple runtime used by AirCard. Upstream reports show that these combinations can break AirTraffic even when scanning works.
- For Microsoft Store iTunes, check the CoreFP warning. The application does not attempt a registry repair because that changes system configuration.
- Keep the iPhone unlocked and connected directly by USB while isolating transfer errors. Wi-Fi pairing can be enabled after USB succeeds.
- If Mode C also fails at byte offset zero, the problem is likely the Apple transport/runtime rather than artwork size or the Wallet card hash.

## Compatibility and limitations

The project obtains iOS version and build information for diagnostics but has no hard-coded future-iOS allowlist. It cannot guarantee private Apple services on a newly released iOS build. The repository's automated tests cover archive construction, chunking, partial writes, and native-error reporting. A physical iPhone is required to verify device-side StreamingZip, AFC read-back, and AirTraffic behavior for a specific driver and iOS combination.

## Security and data handling

AirCard communicates with a paired device through Apple services. Keep the device trusted only on PCs you control. Backups and logs are local. Do not share logs containing card hashes unless you are comfortable disclosing those identifiers.

## Credits

The transport design is based on [airlift](https://github.com/0xjohnnydev/airlift) by 0xjohnny. Passcode-theme compatibility is inspired by Cowabunga and Nugget.
