public class LG {
    private String name; private int size; private java.util.List<String> tags;
    LG name(String n){ this.name = n != null ? n : "default"; return this; }       // null 安全三元 setter（链式）
    LG size(int s){ if(s < 0){ throw new IllegalArgumentException("neg"); } this.size = s; return this; }  // 守卫 setter
    LG tags(java.util.List<String> t){ this.tags = t == null ? java.util.Collections.emptyList() : t; return this; }
    String build(){ return name + ":" + size + ":" + tags.size(); }
    public static void main(String[] a){ System.out.println(new LG().name(null).size(3).tags(null).build()); System.out.println(new LG().name("x").size(1).tags(java.util.Arrays.asList("a")).build()); }
}
