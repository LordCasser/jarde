package trycatch;

import android.os.IBinder;
import android.os.IInterface;
import android.os.Parcel;
import android.os.RemoteException;

public final class TestTryCatchFinally15 implements IInterface {
	private final IBinder zza;

	public TestTryCatchFinally15(IBinder zza) {
		this.zza = zza;
	}

	protected final Parcel test(int i, Parcel parcel) throws RemoteException {
		Parcel obtain = Parcel.obtain();
		try {
			try {
				this.zza.transact(i, parcel, obtain, 0);
				obtain.readException();
				return obtain;
			} catch (RuntimeException e) {
				obtain.recycle();
				throw e;
			}
		} finally {
			parcel.recycle();
		}
	}
}
