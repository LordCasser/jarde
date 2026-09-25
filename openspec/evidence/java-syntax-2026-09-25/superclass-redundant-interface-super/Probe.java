package p;

interface A { default int m() { return 1; } }
interface B { default int m() { return 2; } }
class Base implements A {}

class Child extends Base implements A, B {
    public int m() { return B.super.m(); }
    int call() { return B.super.m(); }
}

class Runner {
    public static void main(String[] args) {
        System.out.println(new Child().call());
    }
}
