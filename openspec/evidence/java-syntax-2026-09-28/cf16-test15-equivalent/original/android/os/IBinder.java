package android.os;

public interface IBinder {
	boolean transact(int code, Parcel input, Parcel output, int flags) throws RemoteException;
}
