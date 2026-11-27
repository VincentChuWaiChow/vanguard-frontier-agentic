# Result 05: Docker and macOS native gate

## Summary

Normal Docker containers cannot provide the project's macOS ARM64/AppKit/Metal
native evidence from a Linux host. Docker-OSX does exist; it wraps a QEMU/KVM
virtual machine rather than turning Linux containers into macOS containers.
Its documented x86_64 host requirement and GPU-acceleration TODO do not establish
the required ARM64/Metal native bootstrap. Do not call all macOS-in-Docker
categorically impossible or substitute such a VM for unexecuted native proof.

Apple documents accelerated graphics/Metal for its virtual Mac device on Apple
Silicon. GitHub's current larger-runner reference lists ARM64 M2 XLarge macOS
runners with GPU hardware acceleration. These are potential approved native-test
routes, not evidence this GPUI build launches there. Account access, billing,
desktop session and actual renderer/test behavior remain unverified. Later human
AT/visual gates are not satisfied by a generic build/runner success.

## Sources

- [Docker container versus VM model](https://docs.docker.com/learn/content/what-is-a-container/)
- [Docker multi-platform builds](https://docs.docker.com/build/building/multi-platform/)
- [Docker-OSX project documentation](https://github.com/sickcodes/Docker-OSX#technical-details)
- [Apple virtualization overview and Metal](https://developer.apple.com/videos/play/wwdc2022/10002/)
- [Apple macOS VM sample](https://developer.apple.com/documentation/virtualization/running-macos-in-a-virtual-machine-on-apple-silicon)
- [GitHub larger-runner specifications](https://docs.github.com/en/actions/reference/runners/larger-runners)

## Verification and limits

One bounded native agent researched current primary documentation; parent opened
the Docker, Docker-OSX, Apple and GitHub pages to corroborate key distinctions.
This is documentation research only. No Docker execution, emulator installation,
cloud rental, billing change, remote CI dispatch or Mac runtime test occurred.
User confirms no Mac available. Existing macOS native gate remains unverified.
