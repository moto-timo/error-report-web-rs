-- Initial database schema for Yocto Project Error Reporting System
-- Compatible with PostgreSQL

-- Create error_reports table
CREATE TABLE error_reports (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    machine VARCHAR(100) NOT NULL,
    distro VARCHAR(100) NOT NULL,
    distro_version VARCHAR(50) NOT NULL,
    build_sys VARCHAR(100) NOT NULL,
    nativelsbstring VARCHAR(100) NOT NULL,
    target_sys VARCHAR(100) NOT NULL,
    failure_task VARCHAR(200) NOT NULL,
    failure_package VARCHAR(200) NOT NULL,
    error_type VARCHAR(100) NOT NULL,
    error_details TEXT NOT NULL,
    log_data TEXT NOT NULL,
    submitter_name VARCHAR(100),
    submitter_email VARCHAR(200),
    bugzilla_link VARCHAR(500),
    branch_commit VARCHAR(100) NOT NULL
);

-- Create build_configurations table
CREATE TABLE build_configurations (
    id SERIAL PRIMARY KEY,
    error_report_id INTEGER NOT NULL REFERENCES error_reports(id) ON DELETE CASCADE,
    bb_version VARCHAR(50) NOT NULL,
    tune_features VARCHAR(200),
    target_fpu VARCHAR(50),
    meta_layers TEXT -- JSON string containing layer information
);

-- Create indexes for performance
CREATE INDEX idx_error_reports_created_at ON error_reports(created_at);
CREATE INDEX idx_error_reports_machine ON error_reports(machine);
CREATE INDEX idx_error_reports_distro ON error_reports(distro);
CREATE INDEX idx_error_reports_distro_version ON error_reports(distro_version);
CREATE INDEX idx_error_reports_error_type ON error_reports(error_type);
CREATE INDEX idx_error_reports_failure_package ON error_reports(failure_package);
CREATE INDEX idx_error_reports_failure_task ON error_reports(failure_task);
CREATE INDEX idx_error_reports_submitter_email ON error_reports(submitter_email);
CREATE INDEX idx_error_reports_branch_commit ON error_reports(branch_commit);

-- Index for build configurations
CREATE INDEX idx_build_configurations_error_report_id ON build_configurations(error_report_id);
CREATE INDEX idx_build_configurations_bb_version ON build_configurations(bb_version);

-- Create full-text search indexes for error details and log data
CREATE INDEX idx_error_reports_error_details_gin ON error_reports USING gin(to_tsvector('english', error_details));
CREATE INDEX idx_error_reports_log_data_gin ON error_reports USING gin(to_tsvector('english', log_data));

-- Comments for documentation
COMMENT ON TABLE error_reports IS 'Store build error reports from Yocto Project builds';
COMMENT ON TABLE build_configurations IS 'Store build configuration details associated with error reports';

COMMENT ON COLUMN error_reports.id IS 'Unique identifier for each error report';
COMMENT ON COLUMN error_reports.created_at IS 'Timestamp when the error was submitted';
COMMENT ON COLUMN error_reports.machine IS 'Target machine (e.g., qemux86-64)';
COMMENT ON COLUMN error_reports.distro IS 'Distribution name (e.g., poky)';
COMMENT ON COLUMN error_reports.distro_version IS 'Distribution version';
COMMENT ON COLUMN error_reports.build_sys IS 'Build system information';
COMMENT ON COLUMN error_reports.nativelsbstring IS 'Native LSB string of the build host';
COMMENT ON COLUMN error_reports.target_sys IS 'Target system triple';
COMMENT ON COLUMN error_reports.failure_task IS 'BitBake task that failed (e.g., do_compile)';
COMMENT ON COLUMN error_reports.failure_package IS 'Package/recipe that failed';
COMMENT ON COLUMN error_reports.error_type IS 'Type/category of error';
COMMENT ON COLUMN error_reports.error_details IS 'Detailed error message';
COMMENT ON COLUMN error_reports.log_data IS 'Complete build log data';
COMMENT ON COLUMN error_reports.submitter_name IS 'Optional name of person submitting the error';
COMMENT ON COLUMN error_reports.submitter_email IS 'Optional email of person submitting the error';
COMMENT ON COLUMN error_reports.bugzilla_link IS 'Optional link to associated Bugzilla entry';
COMMENT ON COLUMN error_reports.branch_commit IS 'Git branch/commit information';

COMMENT ON COLUMN build_configurations.bb_version IS 'BitBake version used for the build';
COMMENT ON COLUMN build_configurations.tune_features IS 'CPU tuning features';
COMMENT ON COLUMN build_configurations.target_fpu IS 'Target FPU configuration';
COMMENT ON COLUMN build_configurations.meta_layers IS 'JSON array of meta layer information';

-- Create a view for error summary statistics
CREATE VIEW error_summary_stats AS
SELECT
    COUNT(*) as total_errors,
    COUNT(DISTINCT machine) as unique_machines,
    COUNT(DISTINCT distro) as unique_distros,
    COUNT(DISTINCT error_type) as unique_error_types,
    COUNT(DISTINCT failure_package) as unique_packages,
    DATE_TRUNC('day', created_at) as error_date,
    COUNT(*) OVER (PARTITION BY DATE_TRUNC('day', created_at)) as daily_count
FROM error_reports
GROUP BY DATE_TRUNC('day', created_at)
ORDER BY error_date DESC;

-- Create a view for recent error trends
CREATE VIEW recent_error_trends AS
SELECT
    DATE_TRUNC('week', created_at) as week_start,
    COUNT(*) as weekly_count,
    COUNT(DISTINCT machine) as unique_machines_per_week,
    COUNT(DISTINCT error_type) as unique_error_types_per_week
FROM error_reports
WHERE created_at >= NOW() - INTERVAL '8 weeks'
GROUP BY DATE_TRUNC('week', created_at)
ORDER BY week_start DESC;

-- Insert some sample data for testing (optional)
-- Uncomment the following lines to add test data

/*
INSERT INTO error_reports (
    machine, distro, distro_version, build_sys, nativelsbstring, target_sys,
    failure_task, failure_package, error_type, error_details, log_data,
    submitter_name, submitter_email, branch_commit
) VALUES
(
    'qemux86-64', 'poky', '4.0', 'x86_64-linux', 'ubuntu-20.04', 'x86_64-poky-linux',
    'do_compile', 'example-package', 'CompilationError',
    'ERROR: example-package-1.0-r0 do_compile: Function failed: do_compile',
    'Log file contents would go here...',
    'Test User', 'test@example.com', 'master:abc123def456'
),
(
    'qemuarm', 'poky', '4.0', 'x86_64-linux', 'fedora-35', 'arm-poky-linux-gnueabi',
    'do_fetch', 'another-package', 'FetchError',
    'ERROR: another-package-2.1-r0 do_fetch: Fetcher failure',
    'Detailed fetch error log...',
    'Another User', 'user@company.com', 'master:def456ghi789'
);

INSERT INTO build_configurations (
    error_report_id, bb_version, tune_features, target_fpu, meta_layers
) VALUES
(
    1, '1.52.0', 'm64 core2', '',
    '[{"name": "meta", "path": "/path/to/meta", "commit": "abc123"}, {"name": "meta-yocto", "path": "/path/to/meta-yocto", "commit": "def456"}]'
),
(
    2, '1.52.0', 'arm armv7a neon', 'hard',
    '[{"name": "meta", "path": "/path/to/meta", "commit": "ghi789"}]'
);
*/
