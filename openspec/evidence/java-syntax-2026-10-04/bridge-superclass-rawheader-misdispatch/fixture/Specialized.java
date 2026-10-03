class Specialized extends Box<String> {
    void set(String v) { System.out.println("SPEC.set(String) ran"); }
    String get() { System.out.println("SPEC.get ran"); return "S"; }
}
