public class DC extends java.lang.Object {
    private java.lang.String tag;

    public DC() {
        super();
        this.tag = "outer";
        return;
    }

    java.lang.String runAll() {
        DC$Mid local1 = new DC$Mid(this);
        local1.leaf().run();
        return new java.lang.StringBuilder().append("done:").append(this.tag).toString();
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) new DC().runAll());
        return;
    }

    static java.lang.String access$000(DC arg0) {
        return arg0.tag;
    }
}

class DC$Mid extends java.lang.Object {
    private int depth;

    final DC this$0;

    DC$Mid(DC arg1) {
        super();
        this.this$0 = arg1;
        this.depth = 1;
        return;
    }

    java.lang.Runnable leaf() {
        return new DC$Mid$Leaf(this);
    }

    static int access$100(DC$Mid arg0) {
        return arg0.depth;
    }
}

class DC$Mid$Leaf extends java.lang.Object implements java.lang.Runnable {
    final DC$Mid this$1;

    DC$Mid$Leaf(DC$Mid arg1) {
        super();
        this.this$1 = arg1;
        return;
    }

    public void run() {
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) DC.access$000(this.this$1.this$0)).append("/").append(DC$Mid.access$100(this.this$1)).append("/").append((java.lang.String) DC.access$000(this.this$1.this$0)).toString());
        return;
    }
}
