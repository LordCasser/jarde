public class TM {
    abstract static class Task {                          // 抽象类骨架
        private final String name;
        protected Task(String name){ this.name = name; }
        abstract void step();                             // 钩子
        protected void hook(){ System.out.println("default-hook"); }   // 可选钩子（被子类覆写）
        public final void run(){                          // final 模板方法
            System.out.println("begin:" + name);
            step();                                       // 虚分派到子类
            hook();
            System.out.println("end");
        }
        String label(){ return name; }
    }
    static class Fast extends Task {
        Fast(String n){ super(n); }
        @Override void step(){ System.out.println("fast-step"); }
    }
    static class Slow extends Task {
        Slow(String n){ super(n); }
        @Override void step(){ System.out.println("slow-step"); }
        @Override protected void hook(){ System.out.println("slow-hook:" + label()); }
    }
    public static void main(String[] a){ new Fast("F").run(); new Slow("S").run(); }
}
