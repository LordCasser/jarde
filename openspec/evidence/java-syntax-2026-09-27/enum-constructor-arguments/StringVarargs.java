public enum StringVarargs {
    PAIR(".dex", ".class"), SINGLE(".xml"), EMPTY;
    private final String[] exts;
    StringVarargs(String... extensions) { this.exts = extensions; }
    public String[] valuesCopy() { return exts; }
}
