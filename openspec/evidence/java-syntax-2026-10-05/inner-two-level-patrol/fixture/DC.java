public class DC {
    private String tag = "outer";
    class Mid {                                             // 第一层内部类
        private int depth = 1;
        class Leaf implements Runnable {                    // 第二层内部类（Mid 内嵌）
            public void run(){
                System.out.println(tag + "/" + depth + "/" + DC.this.tag);   // 双层逃逸：Mid.this.depth + DC.this.tag
            }
        }
        Runnable leaf(){ return new Leaf(); }
    }
    String runAll(){
        Mid m = new Mid();
        m.leaf().run();
        return "done:" + tag;
    }
    public static void main(String[] a){ System.out.println(new DC().runAll()); }
}
