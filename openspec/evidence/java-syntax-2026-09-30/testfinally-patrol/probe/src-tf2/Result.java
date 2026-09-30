// The top-level result the probe's `test` constructs and returns inside the protected body: the
// construction the certificate proves stays inside the body on every side.
public class Result {
    private final int mCode;

    public Result(int code) {
        mCode = code;
    }

    public int getCode() {
        return mCode;
    }

    @Override
    public String toString() {
        return Integer.toString(getCode());
    }
}
