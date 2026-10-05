class IC extends java.lang.Object implements IB {
    static final int LOCAL_CONST = 20;

    IC() {
        super();
        return;
    }

    int viaInherit() {
        return 3;
    }

    int viaNested() {
        return 10;
    }

    int viaDirect() {
        return 1;
    }

    public static void main(java.lang.String[] arg0) {
        IC local1 = new IC();
        java.lang.System.out.println("" + local1.viaInherit() + "/" + local1.viaNested() + "/" + local1.viaDirect() + "/" + 20);
        return;
    }
}
