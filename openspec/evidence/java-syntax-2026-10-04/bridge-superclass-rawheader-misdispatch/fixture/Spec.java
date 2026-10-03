public class Spec extends Outer.Box<String> {      // 嵌套泛型父类（binary 名含 $）
    void set(String v) { System.out.println("SPEC.set(String) ran"); }
    String get() { System.out.println("SPEC.get ran"); return "S"; }
}
