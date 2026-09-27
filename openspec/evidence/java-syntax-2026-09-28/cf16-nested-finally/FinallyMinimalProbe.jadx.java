package jadx.tests.integration.trycatch;

/* JADX INFO: loaded from: FinallyMinimalProbe.class */
public class FinallyMinimalProbe {
    private StringBuilder sb;

    public void test1(int excType) {
        try {
            try {
                call(excType);
            } catch (NullPointerException e) {
                this.sb.append("-catch");
            }
            this.sb.append("-out");
        } finally {
            this.sb.append("-finally");
        }
    }

    public void test2(int excType) {
        try {
            try {
                call(excType);
            } catch (NullPointerException e) {
                this.sb.append("-catch");
            }
        } finally {
            this.sb.append("-finally");
        }
    }

    public void test3(int excType) {
        try {
            call(excType);
        } catch (NullPointerException e) {
            this.sb.append("-catch");
        } finally {
            this.sb.append("-finally");
        }
    }

    public void call(int excType) {
        this.sb.append("call");
        switch (excType) {
            case 1:
                this.sb.append("-npe");
                throw new NullPointerException();
            case 2:
                this.sb.append("-iae");
                throw new IllegalArgumentException();
            default:
                return;
        }
    }
}
