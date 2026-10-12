package net.withforesight.roomtemp.migrationprobe;
import android.app.Instrumentation;
import android.app.Activity;
import android.content.Context;
import android.os.Bundle;
public final class MigrationProbe extends Instrumentation {
    private String mode;
    private static native String runProbe(Context context, String dataDirectory, String mode);
    @Override public void onCreate(Bundle arguments) {
        super.onCreate(arguments);
        mode = arguments.getString("mode", "migrate");
        start();
    }
    @Override public void onStart() {
        Bundle result = new Bundle();
        try {
            System.loadLibrary("roomtemp_migration_probe");
            Context target = getTargetContext();
            String report = runProbe(target, target.getDataDir().getAbsolutePath(), mode);
            result.putString("migration_report", report);
            finish(report != null && report.startsWith("PASS ") ? Activity.RESULT_OK : Activity.RESULT_CANCELED, result);
        } catch (Throwable failure) {
            result.putString("migration_report", "FAIL java=" + failure.getClass().getSimpleName());
            finish(Activity.RESULT_CANCELED, result);
        }
    }
}
