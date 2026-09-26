package defpackage;

/* JADX INFO: loaded from: StringVarargs.class */
public enum StringVarargs {
    PAIR(".dex", ".class"),
    SINGLE(".xml"),
    EMPTY(new String[0]);

    private final String[] exts;

    StringVarargs(String... extensions) {
        this.exts = extensions;
    }

    public String[] valuesCopy() {
        return this.exts;
    }
}
