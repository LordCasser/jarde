package trycatch;

import android.content.Context;
import java.io.IOException;
import java.io.InputStream;
import java.util.Scanner;
import log.DebugLogger;

/* JADX INFO: loaded from: /Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/smali/trycatch/TestTryCatchFinally10.smali */
public class TestTryCatchFinally10 {
    private static final DebugLogger l = null;

    public static String test(Context context, int i) {
        CommonContracts.requireNonNull(context);
        InputStream inputStreamOpenRawResource = null;
        try {
            inputStreamOpenRawResource = context.getResources().openRawResource(i);
            Scanner scannerUseDelimiter = new Scanner(inputStreamOpenRawResource).useDelimiter("\\A");
            return scannerUseDelimiter.hasNext() ? scannerUseDelimiter.next() : "";
        } finally {
            if (inputStreamOpenRawResource != null) {
                try {
                    inputStreamOpenRawResource.close();
                } catch (IOException e) {
                    l.logException(DebugLogger.LogLevel.ERROR, e);
                }
            }
        }
    }
}
