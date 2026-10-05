public class RL {
    String label;
    RL(String l){ label = l; }
    static String selfElem(RL[] xs){ return xs[0].label; }        // 当前类数组元素字段
    static class Inner { String tag; Inner(String t){ tag=t; } }
    static String innerElem(Inner[] xs){ return xs[0].tag; }      // 伴生类数组元素字段（对照）
    public static void main(String[] a){ System.out.println(""+selfElem(new RL[]{new RL("s")})+"/"+innerElem(new Inner[]{new Inner("i")})); }
}
