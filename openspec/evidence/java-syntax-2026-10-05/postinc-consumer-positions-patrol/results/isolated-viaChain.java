public class CH {
    static java.lang.StringBuilder sb = new java.lang.StringBuilder();
    static java.lang.String viaChain(){ int local0 = 5; java.lang.StringBuilder saved0 = CH.sb.append("n"); local0 = local0 + 1; return new java.lang.StringBuilder().append((java.lang.String) CH.sb.toString()).append(":").append(local0).toString(); }
    public static void main(String[] a){ System.out.println(viaChain()); }
}
