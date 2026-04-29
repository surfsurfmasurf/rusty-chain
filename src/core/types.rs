use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    pub prev_hash: String,
    pub timestamp_ms: u64,
    pub nonce: u64,
    pub merkle_root: String,
}

impl BlockHeader {
    pub fn hash(&self) -> String {
        crate::core::hash::header_hash(self)
    }

    /// Optimized hash check for difficulty.
    pub fn has_valid_difficulty(&self, difficulty: u32) -> bool {
        self.verify_pow(difficulty).is_ok()
    }

    /// Stateless header verification (PoW check).
    pub fn verify_pow(&self, difficulty: u32) -> anyhow::Result<()> {
        let hash = self.hash();
        let target = "0".repeat(difficulty as usize);
        if !hash.starts_with(&target) {
            anyhow::bail!("invalid PoW: hash={} difficulty={}", hash, difficulty);
        }
        Ok(())
    }
}

/// A minimal transaction (Week 2: add optional signatures).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    #[serde(default)]
    pub fee: u64,
    pub nonce: u64,

    /// Optional ed25519 public key (hex) used to verify `signature_b64`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_hex: Option<String>,

    /// Optional ed25519 signature (base64) over the signing payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_b64: Option<String>,

    /// Optional comment/metadata for the transaction (limit: 128 chars)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Optional sequence number for the transaction (future-proofing)
    #[serde(default)]
    pub sequence: u32,

    /// Optional timestamp for when the transaction was created (Unix epoch ms)
    #[serde(default)]
    pub timestamp_ms: u64,

    /// Optional locktime (block height). If set, the transaction is invalid until the chain reaches this height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locktime: Option<u64>,

    /// Optional expiry (block height). If set, the transaction is invalid after the chain reaches this height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,

    /// Optional priority level (0-255). Used for mempool ordering and processing.
    #[serde(default)]
    pub priority: u8,

    /// Is the transaction private? (Future proofing for zero-knowledge or encrypted txs)
    #[serde(default)]
    pub is_private: bool,

    /// Optional time-to-live (milliseconds) for mempool duration.
    #[serde(default)]
    pub ttl_ms: u64,

    /// UNIQUE: Unique identifier for the transaction (UUID v4), used for tracking through P2P and mempool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce_id: Option<String>,

    /// Transaction expiration timestamp (Unix epoch ms).
    /// If non-zero, the transaction is invalid if `now_ms > expiration_ms`.
    #[serde(default)]
    pub expiration_ms: u64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p2p_message_id: Option<String>,

    /// Hierarchical network identifiers for routing and scalability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shard_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_id: Option<String>,

    /// Multi-layered network identifiers for Day 68.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric_id: Option<String>,

    /// Geographical network identifiers for routing and scalability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,

    /// Checkpoint identifier for transaction anchoring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rack_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hop_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,

    /// Network topology identifiers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane_id: Option<String>,

    /// Unique transaction nonce hash (UUID/Nonce) for double-spend protection at P2P level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_id: Option<String>,

    /// Checksum of the transaction payload for quick integrity verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_checksum: Option<String>,

    /// Optional P2P message ID to handle P2P-level deduplication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    /// Optional transaction tag for categorizing or filtering transactions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Checkpoint index to track when this transaction was last verified against a checkpoint.
    #[serde(default)]
    pub checkpoint_index: u32,

    /// Is the transaction a part of a batch?
    #[serde(default)]
    pub is_batched: bool,

    /// Reference to a parent transaction (for nested transactions or batch linkage).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Transaction metadata schema identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<String>,

    /// Optional transaction difficulty for local or specific verification.
    #[serde(default)]
    pub local_difficulty: u32,

    /// Unique nonce for preventing same-block replays.
    #[serde(default)]
    pub salt: u64,

    /// Optional transaction size in bytes (calculated or fixed).
    #[serde(default)]
    pub size_bytes: u32,

    /// Unique request ID for correlating P2P requests and responses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Transaction origin (e.g. "wallet", "faucet", "node").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,

    /// Optional reference to a linked transaction in another chain or system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,

    /// Transactionclassification (e.g. "payment", "smart_contract", "vote").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Optional transaction priority score (0.0 to 1.0).
    #[serde(default)]
    pub priority_score: f64,

    /// Optional transaction risk score (0.0 to 1.0).
    #[serde(default)]
    pub risk_score: f64,

    /// Optional reference to a specific regulatory jurisdiction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction_id: Option<String>,

    /// Transaction weight for congestion control.
    #[serde(default)]
    pub weight: u32,

    /// Is the transaction valid for mining? (e.g. used for test transactions)
    #[serde(default)]
    pub is_minable: bool,

    /// Optional reference to a previous state hash for verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_ref: Option<String>,

    /// Transaction expiration status for pruning.
    #[serde(default)]
    pub is_expired: bool,

    /// Is the transaction valid for replay?
    #[serde(default)]
    pub is_replayable: bool,

    /// Optional gas limit for transaction execution.
    #[serde(default)]
    pub gas_limit: u64,

    /// Optional reference to an external system for validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_system: Option<String>,

    /// Transaction-specific logic or script for advanced validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,

    /// Optional identifier for a cross-chain bridge transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_id: Option<String>,

    /// Optional reference to a specific application or contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,

    /// Optional transaction label (e.g. "gift", "work", "internal").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Transaction specific status flags.
    #[serde(default)]
    pub flags: u32,

    /// Core: Added 'label' field to Transaction and TxSignPayload for categorization.
    /// This field is optional and can be used for various purposes like accounting or filtering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_label: Option<String>,

    /// Unique P2P session identifier to prevent cross-session replay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Optional pool identifier for mempool isolation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pool_id: Option<String>,

    /// Optional trace identifier for transaction lifecycle monitoring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Optional reference to a specific event or alert ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,

    /// Optional reference to a specific log or audit entry ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_id: Option<String>,

    /// Optional reference to a specific source system or node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,

    /// Optional reference to a specific data center or location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,

    /// Optional reference to a specific data center rack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datacenter_id: Option<String>,

    /// Optional reference to a specific storage or data partition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partition_id: Option<String>,

    /// Optional reference to a specific security or compliance domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<String>,

    /// Optional reference to a specific regulatory or legal framework.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_id: Option<String>,

    /// Version number for the transaction format.
    #[serde(default = "default_tx_version")]
    pub version: u32,

    /// Unique hash for transaction deduplication and identification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id_hash: Option<String>,

    /// Optional reference to a specific validation node or authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_id: Option<String>,

    /// Transaction reference for specific contract or logic instantiation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instantiation_id: Option<String>,

    /// Transaction reference for specific module or plugin linkage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,

    /// Transaction reference for specific plugin or extension identification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,

    /// Optional reference to a specific service or backend identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,

    /// Transaction reference for specific API or endpoint identification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,

    /// Hierarchical workflow identifiers for transaction lifecycle tracking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,

    /// Enhanced network and system identifiers for deep observability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// Transaction lifecycle state and status flags.
    #[serde(default)]
    pub is_reverting: bool,
    #[serde(default)]
    pub is_conditional: bool,
    #[serde(default)]
    pub is_delegated: bool,

    /// Extended audit identifiers for compliance and regulatory tracking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compliance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_ref: Option<String>,

    /// Transaction security and integrity fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secure_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_id: Option<String>,

    /// Future scalability and state management fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_commitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,

    /// Future cross-chain and interoperability fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_chain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_chain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_adapter_id: Option<String>,

    /// Transaction metadata and labeling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,

    /// Future account and permission management fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_set: Option<String>,

    /// Transaction resource and cost tracking.
    #[serde(default)]
    pub compute_units: u64,
    #[serde(default)]
    pub storage_units: u64,
    #[serde(default)]
    pub bandwidth_units: u64,

    /// New fields for Day 67: advanced scaling and governance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote_weight: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity_proof: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Future scalability and network performance fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub throughput_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reliability_id: Option<String>,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            from: "".to_string(),
            to: "".to_string(),
            amount: 0,
            fee: 0,
            nonce: 0,
            pubkey_hex: None,
            signature_b64: None,
            memo: None,
            sequence: 0,
            timestamp_ms: crate::core::time::now_ms(),
            locktime: None,
            expiry: None,
            priority: 0,
            ttl_ms: 0,
            nonce_id: None,
            expiration_ms: 0,
            p2p_message_id: None,
            shard_id: None,
            subnet_id: None,
            cluster_id: None,
            cell_id: None,
            area_id: None,
            fabric_id: None,
            anchor_id: None,
            node_id: None,
            rack_id: None,
            slot_id: None,
            container_id: None,
            process_id: None,
            thread_id: None,
            deployment_id: None,
            environment_id: None,
            org_id: None,
            region_id: None,
            zone_id: None,
            group_id: None,
            path_id: None,
            hop_id: None,
            route_id: None,
            layer_id: None,
            tier_id: None,
            plane_id: None,
            message_id: None,
            tag: None,
            unique_id: None,
            checkpoint_index: 0,
            is_batched: false,
            parent_id: None,
            schema_id: None,
            local_difficulty: 0,
            salt: 0,
            size_bytes: 0,
            request_id: None,
            origin: None,
            external_ref: None,
            category: None,
            priority_score: 0.0,
            risk_score: 0.0,
            jurisdiction_id: None,
            weight: 0,
            is_minable: true,
            state_ref: None,
            is_expired: false,
            is_replayable: false,
            gas_limit: 0,
            external_system: None,
            script: None,
            bridge_id: None,
            app_id: None,
            label: None,
            metadata_label: None,
            flags: 0,
            is_private: false,
            session_id: None,
            pool_id: None,
            trace_id: None,
            event_id: None,
            audit_id: None,
            source_id: None,
            location_id: None,
            datacenter_id: None,
            partition_id: None,
            domain_id: None,
            framework_id: None,
            payload_checksum: None,
            tx_id_hash: None,
            authority_id: None,
            instantiation_id: None,
            module_id: None,
            plugin_id: None,
            service_id: None,
            endpoint_id: None,
            version: 1,
            flow_id: None,
            step_id: None,
            task_id: None,
            sequence_id: None,
            stream_id: None,
            batch_id: None,
            controller_id: None,
            worker_id: None,
            instance_id: None,
            is_reverting: false,
            is_conditional: false,
            is_delegated: false,
            compliance_id: None,
            policy_id: None,
            legal_ref: None,
            secure_hash: None,
            bundle_id: None,
            proof_id: None,
            state_commitment: None,
            witness_id: None,
            snapshot_id: None,
            origin_chain: None,
            target_chain: None,
            bridge_adapter_id: None,
            topic: None,
            sub_topic: None,
            channel_id: None,
            actor_id: None,
            role_id: None,
            permission_set: None,
            compute_units: 0,
            storage_units: 0,
            bandwidth_units: 0,
            governance_id: None,
            proposal_id: None,
            vote_weight: None,
            encryption_key_id: None,
            integrity_proof: None,
            recovery_id: None,
            metadata_uri: None,
            content_hash: None,
            mime_type: None,
            latency_id: None,
            throughput_id: None,
            reliability_id: None,
        }
    }
}

fn default_tx_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxSignPayload {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(default)]
    pub sequence: u32,
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locktime: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub ttl_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce_id: Option<String>,
    #[serde(default)]
    pub expiration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p2p_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shard_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rack_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot_id: Option<String>,

    /// UNIQUE: Container identifier for virtualized node tracking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default)]
    pub checkpoint_index: u32,
    #[serde(default)]
    pub is_batched: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<String>,
    #[serde(default)]
    pub local_difficulty: u32,
    #[serde(default)]
    pub salt: u64,
    #[serde(default)]
    pub size_bytes: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub priority_score: f64,
    #[serde(default)]
    pub risk_score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction_id: Option<String>,
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub is_minable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_ref: Option<String>,
    #[serde(default)]
    pub is_expired: bool,
    #[serde(default)]
    pub is_replayable: bool,
    #[serde(default)]
    pub gas_limit: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_label: Option<String>,
    #[serde(default)]
    pub flags: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pool_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datacenter_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partition_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_id: Option<String>,
    #[serde(default)]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instantiation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(default)]
    pub is_reverting: bool,
    #[serde(default)]
    pub is_conditional: bool,
    #[serde(default)]
    pub is_delegated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compliance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secure_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_commitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_chain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_chain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_adapter_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_set: Option<String>,
    #[serde(default)]
    pub compute_units: u64,
    #[serde(default)]
    pub storage_units: u64,
    #[serde(default)]
    pub bandwidth_units: u64,

    /// New fields for Day 67: advanced scaling and governance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote_weight: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity_proof: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub throughput_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reliability_id: Option<String>,
}

impl Transaction {
    pub fn new(from: impl Into<String>, to: impl Into<String>, amount: u64, nonce: u64) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee: 0,
            nonce,
            ..Default::default()
        }
    }

    pub fn new_with_sequence(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        nonce: u64,
        sequence: u32,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee: 0,
            nonce,
            sequence,
            ..Default::default()
        }
    }

    pub fn new_with_locktime(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        nonce: u64,
        locktime: u64,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee: 0,
            nonce,
            locktime: Some(locktime),
            ..Default::default()
        }
    }

    pub fn new_with_fee(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        fee: u64,
        nonce: u64,
        sequence: u32,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            fee,
            nonce,
            sequence,
            ..Default::default()
        }
    }

    pub fn signing_payload(&self) -> TxSignPayload {
        TxSignPayload {
            from: self.from.clone(),
            to: self.to.clone(),
            amount: self.amount,
            fee: self.fee,
            nonce: self.nonce,
            memo: self.memo.clone(),
            sequence: self.sequence,
            timestamp_ms: self.timestamp_ms,
            locktime: self.locktime,
            expiry: self.expiry,
            priority: self.priority,
            is_private: self.is_private,
            ttl_ms: self.ttl_ms,
            nonce_id: self.nonce_id.clone(),
            expiration_ms: self.expiration_ms,
            p2p_message_id: self.p2p_message_id.clone(),
            shard_id: self.shard_id.clone(),
            subnet_id: self.subnet_id.clone(),
            cluster_id: self.cluster_id.clone(),
            cell_id: self.cell_id.clone(),
            area_id: self.area_id.clone(),
            fabric_id: self.fabric_id.clone(),
            anchor_id: self.anchor_id.clone(),
            node_id: self.node_id.clone(),
            rack_id: self.rack_id.clone(),
            slot_id: self.slot_id.clone(),
            container_id: self.container_id.clone(),
            process_id: self.process_id.clone(),
            thread_id: self.thread_id.clone(),
            deployment_id: self.deployment_id.clone(),
            environment_id: self.environment_id.clone(),
            org_id: self.org_id.clone(),
            region_id: self.region_id.clone(),
            zone_id: self.zone_id.clone(),
            group_id: self.group_id.clone(),
            layer_id: self.layer_id.clone(),
            tier_id: self.tier_id.clone(),
            plane_id: self.plane_id.clone(),
            unique_id: self.unique_id.clone(),
            payload_checksum: self.payload_checksum.clone(),
            message_id: self.message_id.clone(),
            tag: self.tag.clone(),
            checkpoint_index: self.checkpoint_index,
            is_batched: self.is_batched,
            parent_id: self.parent_id.clone(),
            schema_id: self.schema_id.clone(),
            local_difficulty: self.local_difficulty,
            salt: self.salt,
            size_bytes: self.size_bytes,
            request_id: self.request_id.clone(),
            origin: self.origin.clone(),
            external_ref: self.external_ref.clone(),
            category: self.category.clone(),
            priority_score: self.priority_score,
            risk_score: self.risk_score,
            jurisdiction_id: self.jurisdiction_id.clone(),
            weight: self.weight,
            is_minable: self.is_minable,
            state_ref: self.state_ref.clone(),
            is_expired: self.is_expired,
            is_replayable: self.is_replayable,
            gas_limit: self.gas_limit,
            external_system: self.external_system.clone(),
            script: self.script.clone(),
            bridge_id: self.bridge_id.clone(),
            app_id: self.app_id.clone(),
            label: self.label.clone(),
            metadata_label: self.metadata_label.clone(),
            flags: self.flags,
            session_id: self.session_id.clone(),
            pool_id: self.pool_id.clone(),
            trace_id: self.trace_id.clone(),
            event_id: self.event_id.clone(),
            audit_id: self.audit_id.clone(),
            source_id: self.source_id.clone(),
            location_id: self.location_id.clone(),
            datacenter_id: self.datacenter_id.clone(),
            partition_id: self.partition_id.clone(),
            domain_id: self.domain_id.clone(),
            framework_id: self.framework_id.clone(),
            tx_id_hash: self.tx_id_hash.clone(),
            authority_id: self.authority_id.clone(),
            instantiation_id: self.instantiation_id.clone(),
            module_id: self.module_id.clone(),
            plugin_id: self.plugin_id.clone(),
            service_id: self.service_id.clone(),
            endpoint_id: self.endpoint_id.clone(),
            version: self.version,
            flow_id: self.flow_id.clone(),
            step_id: self.step_id.clone(),
            task_id: self.task_id.clone(),
            sequence_id: self.sequence_id.clone(),
            stream_id: self.stream_id.clone(),
            batch_id: self.batch_id.clone(),
            controller_id: self.controller_id.clone(),
            worker_id: self.worker_id.clone(),
            instance_id: self.instance_id.clone(),
            is_reverting: self.is_reverting,
            is_conditional: self.is_conditional,
            is_delegated: self.is_delegated,
            compliance_id: self.compliance_id.clone(),
            policy_id: self.policy_id.clone(),
            legal_ref: self.legal_ref.clone(),
            secure_hash: self.secure_hash.clone(),
            bundle_id: self.bundle_id.clone(),
            proof_id: self.proof_id.clone(),
            state_commitment: self.state_commitment.clone(),
            witness_id: self.witness_id.clone(),
            snapshot_id: self.snapshot_id.clone(),
            origin_chain: self.origin_chain.clone(),
            target_chain: self.target_chain.clone(),
            bridge_adapter_id: self.bridge_adapter_id.clone(),
            topic: self.topic.clone(),
            sub_topic: self.sub_topic.clone(),
            channel_id: self.channel_id.clone(),
            actor_id: self.actor_id.clone(),
            role_id: self.role_id.clone(),
            permission_set: self.permission_set.clone(),
            compute_units: self.compute_units,
            storage_units: self.storage_units,
            bandwidth_units: self.bandwidth_units,
            governance_id: self.governance_id.clone(),
            proposal_id: self.proposal_id.clone(),
            vote_weight: self.vote_weight,
            encryption_key_id: self.encryption_key_id.clone(),
            integrity_proof: self.integrity_proof.clone(),
            recovery_id: self.recovery_id.clone(),
            metadata_uri: self.metadata_uri.clone(),
            content_hash: self.content_hash.clone(),
            mime_type: self.mime_type.clone(),
            latency_id: self.latency_id.clone(),
            throughput_id: self.throughput_id.clone(),
            reliability_id: self.reliability_id.clone(),
        }
    }

    pub fn signing_bytes(&self) -> Vec<u8> {
        // JSON keeps this demo-friendly; if we need canonical encoding later, we can swap it.
        serde_json::to_vec(&self.signing_payload()).expect("serialize signing payload")
    }

    /// Transaction ID (hash)
    pub fn id(&self) -> String {
        crate::core::hash::tx_hash(self)
    }

    /// Get size.
    pub fn size(&self) -> usize {
        serde_json::to_vec(self).unwrap_or_default().len()
    }

    /// Check if the transaction is a coinbase (reward) transaction.
    pub fn is_coinbase(&self) -> bool {
        self.from == "SYSTEM"
    }

    /// Basic sanity checks (Week 1/early Week 2 demo).
    ///
    /// Note: signatures/balances/nonces will be enforced later.
    pub fn validate_basic(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.from.trim().is_empty(), "tx.from must be non-empty");
        anyhow::ensure!(!self.to.trim().is_empty(), "tx.to must be non-empty");
        anyhow::ensure!(self.from != self.to, "tx.from and tx.to must differ");
        // Minimum amount of 1 unit (prevents dust/negative amounts)
        anyhow::ensure!(self.amount > 0, "tx.amount must be > 0");

        // Enhanced amount check for large transactions
        if self.amount > 1_000_000_000 {
            println!("Large transaction detected: {} units", self.amount);
        }

        // Priority score range check
        anyhow::ensure!(
            self.priority_score >= 0.0 && self.priority_score <= 1.0,
            "priority_score must be between 0.0 and 1.0"
        );

        // Risk score range check
        anyhow::ensure!(
            self.risk_score >= 0.0 && self.risk_score <= 1.0,
            "risk_score must be between 0.0 and 1.0"
        );

        // Expiration check
        if self.expiration_ms > 0 {
            let now = crate::core::time::now_ms();
            anyhow::ensure!(
                now < self.expiration_ms,
                "transaction has expired (now={} expiration={})",
                now,
                self.expiration_ms
            );
        }

        if let Some(memo) = &self.memo {
            anyhow::ensure!(memo.len() <= 128, "memo must be <= 128 characters");
        }

        // Validate metadata_label length if present
        if let Some(label) = &self.metadata_label {
            anyhow::ensure!(label.len() <= 64, "metadata_label must be <= 64 characters");
        }

        anyhow::ensure!(self.version > 0, "tx.version must be > 0");

        if let Some(uid) = &self.unique_id {
            anyhow::ensure!(
                !uid.trim().is_empty(),
                "tx.unique_id must not be empty if present"
            );
        }

        if let Some(sid) = &self.session_id {
            anyhow::ensure!(
                !sid.trim().is_empty(),
                "tx.session_id must not be empty if present"
            );
        }

        if let Some(pc) = &self.payload_checksum {
            anyhow::ensure!(
                !pc.trim().is_empty(),
                "tx.payload_checksum must not be empty if present"
            );
        }

        // Validate multi-layer networking fields if present
        if let Some(cid) = &self.cell_id {
            anyhow::ensure!(
                !cid.trim().is_empty(),
                "tx.cell_id must not be empty if present"
            );
        }

        if let Some(aid) = &self.area_id {
            anyhow::ensure!(
                !aid.trim().is_empty(),
                "tx.area_id must not be empty if present"
            );
        }

        Ok(())
    }

    /// Basic tx validation for accepting into the mempool or a block.
    pub fn validate_accept(&self) -> anyhow::Result<()> {
        self.validate_basic()?;
        self.verify_signature_if_present()?;
        Ok(())
    }

    /// Verify signature if present.
    ///
    /// Rules (for now):
    /// - If both `pubkey_hex` and `signature_b64` are present, verify strictly.
    /// - If neither is present, treat as unsigned and accept.
    /// - If only one is present, reject.
    pub fn verify_signature_if_present(&self) -> anyhow::Result<()> {
        match (&self.pubkey_hex, &self.signature_b64) {
            (None, None) => Ok(()),
            (Some(_), None) | (None, Some(_)) => {
                anyhow::bail!("tx signature fields must be both present or both absent")
            }
            (Some(pk_hex), Some(sig_b64)) => {
                anyhow::ensure!(
                    self.from == *pk_hex,
                    "signed tx must use from=<pubkey_hex> (from={} pubkey_hex={})",
                    self.from,
                    pk_hex
                );

                let vk = crate::core::crypto::verifying_key_from_hex(pk_hex)?;
                crate::core::crypto::verify_bytes(&vk, &self.signing_bytes(), sig_b64)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Transaction>,
}

impl Block {
    pub fn is_coinbase(&self) -> bool {
        self.txs.first().is_some_and(|tx| tx.is_coinbase())
    }

    pub fn total_reward(&self) -> u64 {
        let block_reward = 50;
        let fees: u64 = self
            .txs
            .iter()
            .filter(|tx| !tx.is_coinbase())
            .map(|tx| tx.fee)
            .sum();
        block_reward + fees
    }

    /// Calculate the size of the block in bytes when serialized.
    pub fn size(&self) -> usize {
        serde_json::to_vec(self).unwrap_or_default().len()
    }

    /// Returns true if the block's header satisfies the given PoW difficulty.
    pub fn is_valid_pow(&self, difficulty: u32) -> bool {
        self.header.verify_pow(difficulty).is_ok()
    }

    /// Basic block validation against a previous header.
    pub fn validate_with_prev(
        &self,
        prev_header: &BlockHeader,
        difficulty: u32,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.header.prev_hash == prev_header.hash(),
            "invalid prev_hash: {} (expected {})",
            self.header.prev_hash,
            prev_header.hash()
        );
        anyhow::ensure!(
            self.header.timestamp_ms >= prev_header.timestamp_ms,
            "timestamp cannot go backward: {} (prev: {})",
            self.header.timestamp_ms,
            prev_header.timestamp_ms
        );
        self.header.verify_pow(difficulty)?;
        Ok(())
    }
}
