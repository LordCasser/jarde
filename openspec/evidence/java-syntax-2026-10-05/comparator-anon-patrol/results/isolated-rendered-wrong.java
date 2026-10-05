class BA {
    static class User { final int age; User(int a){ age = a; } public String toString(){ return ""+age; } }
    public BA() {
        super();
        return;
    }

    static java.util.List byAnon(java.util.List arg0) {
        java.util.ArrayList local1 = new java.util.ArrayList((java.util.Collection) arg0);
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        java.util.ArrayList local1 = new java.util.ArrayList();
        local1.add((java.lang.Object) new User(30));
        local1.add((java.lang.Object) new User(10));
        local1.add((java.lang.Object) new User(20));
        java.lang.System.out.println((java.lang.Object) byAnon((java.util.List) local1));
        return;
    }
}
