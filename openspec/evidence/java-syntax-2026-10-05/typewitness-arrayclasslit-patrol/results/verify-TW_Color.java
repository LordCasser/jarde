enum TW$Color {
    RED,
    GREEN,
    BLUE;

    static java.lang.String all() {
        java.lang.StringBuilder local0;
        TW$Color[] local1;
        local0 = new java.lang.StringBuilder();
        local1 = values();
        for (TW$Color local4 : local1) {
            local0.append((java.lang.String) local4.name()).append(",");
        }
        return local0.toString();
    }
}
