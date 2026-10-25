# AppImage update restart review

## Scope

Card 051, based on dev at 90b4406cd768d24006ca24eab454bcd078620ff2. The user approved fixing the restart edge case and releasing the patch. Both review axes ran independently with the code-review skill.

## Initial review

Head: 6aa8d748ab5c448db6c6b4994389c6f5dc169d3a.

Standards: one P2 finding. Failed native-test cleanup depended on SingletonLock even though Muzo does not acquire a single-instance lock. A detached replacement could survive a failed reconnection.

Spec: one P2 finding, the same cleanup problem. The maintained regression must stop its replacement before removing the isolated profile.

No production restart findings were reported.

## Fix re-review

Head: 979734e4db99c5c4ad48637eba6fd29fcedfab8d.

Standards: zero findings. Spec: zero findings.

Cleanup now finds the exact test-owned AppImage processes, including a replacement whose launch arguments were lost, and waits for termination with bounded escalation. A live child-process probe confirmed it terminates the selected player while preserving an unrelated one.

## Regression evidence

The real AppImage test failed before the production change in both extraction and FUSE modes. File replacement succeeded, but the replacement debugging endpoint did not appear. A separate unit regression confirmed the unsafe relaunch call and missing launch arguments.

Both native modes pass with the fresh 0.0.4 installer. They verify original-process exit, replacement window, packaged version, persisted library, byte-for-byte replacement and retained menu path. Expected missing-executable and rejected-path errors pass adapter checks.

Release PR review and native CI evidence are recorded on the pull requests before publication.
