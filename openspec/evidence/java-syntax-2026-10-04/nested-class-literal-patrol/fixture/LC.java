public class LC {
    static class Inner { }
    void m() { class Local { } Object o = Local.class; }
    void n() { Runnable r = new Runnable() { public void run() { } }; Object c = r.getClass(); }
}
