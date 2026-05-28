DROP TABLE subscriptions;

-- Table used by workers to acquire locks for SubscriptionTopic processing
-- to ensure that only one worker is processing a given topic at a time 
-- and to track the last sequence position that was processed for a given topic.
CREATE TABLE
    subscription_topic_locks (
        tenant TEXT NOT NULL,
        project TEXT NOT NULL,
        topic TEXT NOT NULL,
        topic_version TEXT NOT NULL,
        lock_sequence_position BIGINT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        PRIMARY KEY (tenant, project, topic),
        -- Foreign keys to ensure that locks are only created for existing tenants, projects, and topics
        FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        FOREIGN KEY (tenant, project) REFERENCES projects (tenant, id) ON DELETE CASCADE,
        FOREIGN KEY (tenant, project, topic_version) REFERENCES resources (tenant, project, version_id) ON DELETE CASCADE
    );

CREATE OR REPLACE FUNCTION update_modified_column()
    RETURNS TRIGGER AS $$
    BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_modified_time BEFORE
UPDATE ON subscription_topic_locks FOR EACH ROW EXECUTE PROCEDURE update_modified_column ();