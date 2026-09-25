package defpackage;
public class DirectArrayCtorRunner { public static void main(String[] args) { for (int n : new int[]{0,3,-1}) { try { System.out.println(DirectArrayCtorRef.make(n).length); } catch (Throwable t) { System.out.println(t.getClass().getName()); } } } }
