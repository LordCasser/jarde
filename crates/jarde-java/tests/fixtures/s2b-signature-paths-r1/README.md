# S2b Signature/path test fixture

The three custom generic classes are copied byte-for-byte from frozen dev-v2 custom-javac23-debug `target.jar`; the ten platform headers are copied byte-for-byte from its JDK 8 generic header closure. SHA-256 and source archive pins are in `manifest.json`. Tests use reader classfile facts and constant-pool parsing, then bind each header to a physical StandaloneRoot identity derived from the same bytes. The tested selection provider keeps app and bootstrap loader IDs separate.

This fixture validates the Signature facts/path helper only. It does not test Engine admission or Builder projection, does not execute target classes, and does not include holdout bytes.
