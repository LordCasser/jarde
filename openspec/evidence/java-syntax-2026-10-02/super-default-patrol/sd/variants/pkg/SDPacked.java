package pkg;
public class SDPacked {
    public interface I { default String name() { return "PI"; } }
    public static class Use implements I {
        @Override public String name() { return I.super.name(); }
    }
    public static void main(String[] x) { System.out.println(new Use().name()); }
}
