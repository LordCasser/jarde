public class CO extends java.lang.Object {
    public CO() {
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        B local1 = new B();
        C local2 = new C();
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append(local1.a).append("/").append(local1.b).append("/").append(local2.a).append("/").append(local2.c).toString());
        return;
    }

    static class B extends A {
        int b;

        B() {
            super();
            this.b = 10;
            return;
        }

        int hook() {
            return this.b + 1;
        }
    }

    static class C extends A {
        int c;

        C() {
            super();
            this.c = 5;
            this.c = this.c + 1;
            return;
        }

        int hook() {
            return 7;
        }
    }

    static abstract class A extends java.lang.Object {
        int a;

        A() {
            super();
            this.a = this.hook();
            return;
        }

        abstract int hook();
    }
}
