package jadx.tests.integration.trycatch;

import jadx.tests.api.utils.assertj.JadxAssertions;

/* JADX INFO: loaded from: SwitchCatchMinimal.class */
public class SwitchCatchMinimal {
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

    public String runTest(int testNumber, int excType) {
        this.sb = new StringBuilder();
        try {
            switch (testNumber) {
                case 1:
                    test1(excType);
                    break;
                case 2:
                    test2(excType);
                    break;
                case 3:
                    test3(excType);
                    break;
            }
        } catch (IllegalArgumentException e) {
            JadxAssertions.assertThat(Integer.valueOf(excType)).isEqualTo(2);
        }
        return this.sb.toString();
    }
}
