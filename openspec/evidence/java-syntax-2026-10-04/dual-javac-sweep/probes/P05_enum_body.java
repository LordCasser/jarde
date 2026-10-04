public enum P05_enum_body {
    A("x"){ String show(){ return "A:"+v; } }, B("y"){ String show(){ return "B:"+v; } };
    final String v; P05_enum_body(String s){ v=s; }
    abstract String show();
    public static void main(String[] a){ System.out.println(A.show()+"/"+B.show()); }
}
