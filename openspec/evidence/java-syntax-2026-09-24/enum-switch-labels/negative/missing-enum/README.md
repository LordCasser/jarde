This boundary uses the original subject, helper and runner class files but omits `Hue.class`.
The JVM is invoked with `-Xverify:all`; class loading fails with `NoClassDefFoundError: Hue`
before the subject can run. This is a missing dependency, not an invalid class-file verifier
result, so a class-source reader must report the unresolved enum dependency and decline the
cross-class label projection.
