/** A nest whose members share private access: the host carries `NestMembers`, the member `NestHost`. */
public class NestSample {
    private int secret = 7;

    public class Inner {
        public int read() {
            return secret;
        }
    }
}
