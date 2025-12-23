# SweetGrass — API Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass exposes three API interfaces:

| Interface | Protocol | Use Case |
|-----------|----------|----------|
| **gRPC** | Protocol Buffers | High-performance internal services |
| **REST** | JSON over HTTP | External integrations, web clients |
| **GraphQL** | GraphQL over HTTP | Flexible queries, provenance graphs |

---

## 2. gRPC API

### 2.1 Service Definition

```protobuf
syntax = "proto3";

package sweetgrass.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

// SweetGrass Provenance Service
service SweetGrass {
    // ==================== Braid Operations ====================
    
    // Create a new Braid
    rpc CreateBraid(CreateBraidRequest) returns (CreateBraidResponse);
    
    // Get Braid by ID
    rpc GetBraid(GetBraidRequest) returns (GetBraidResponse);
    
    // Get Braids for a data hash
    rpc GetBraidsForData(GetBraidsForDataRequest) returns (GetBraidsForDataResponse);
    
    // List Braids with filtering
    rpc ListBraids(ListBraidsRequest) returns (ListBraidsResponse);
    
    // ==================== Provenance Queries ====================
    
    // Get provenance graph for an entity
    rpc GetProvenanceGraph(GetProvenanceGraphRequest) returns (GetProvenanceGraphResponse);
    
    // Get attribution chain
    rpc GetAttributionChain(GetAttributionChainRequest) returns (GetAttributionChainResponse);
    
    // Get top contributors for an entity
    rpc GetTopContributors(GetTopContributorsRequest) returns (GetTopContributorsResponse);
    
    // ==================== Agent Queries ====================
    
    // Get agent's contributions
    rpc GetAgentContributions(GetAgentContributionsRequest) returns (GetAgentContributionsResponse);
    
    // ==================== Compression ====================
    
    // Compress session to Braids
    rpc CompressSession(CompressSessionRequest) returns (CompressSessionResponse);
    
    // Create meta-Braid (summary)
    rpc CreateMetaBraid(CreateMetaBraidRequest) returns (CreateMetaBraidResponse);
    
    // ==================== Anchoring ====================
    
    // Anchor Braid to LoamSpine
    rpc AnchorBraid(AnchorBraidRequest) returns (AnchorBraidResponse);
    
    // Verify Braid anchor
    rpc VerifyAnchor(VerifyAnchorRequest) returns (VerifyAnchorResponse);
    
    // ==================== Search ====================
    
    // Full-text search Braids
    rpc SearchBraids(SearchBraidsRequest) returns (SearchBraidsResponse);
    
    // ==================== Streaming ====================
    
    // Subscribe to new Braids
    rpc SubscribeBraids(SubscribeBraidsRequest) returns (stream Braid);
    
    // ==================== GraphQL ====================
    
    // Execute GraphQL query
    rpc GraphQL(GraphQLRequest) returns (GraphQLResponse);
    
    // ==================== Health ====================
    
    // Health check
    rpc Health(HealthRequest) returns (HealthResponse);
}
```

### 2.2 Message Definitions

```protobuf
// ==================== Braid ====================

message Braid {
    string id = 1;
    string braid_type = 2;
    string data_hash = 3;
    string mime_type = 4;
    uint64 size = 5;
    Activity was_generated_by = 6;
    repeated EntityReference was_derived_from = 7;
    string was_attributed_to = 8;  // DID
    google.protobuf.Timestamp generated_at_time = 9;
    google.protobuf.Struct metadata = 10;
    EcoPrimalsAttributes ecop = 11;
    BraidSignature signature = 12;
    LoamAnchor loam_anchor = 13;
}

message Activity {
    string id = 1;
    string activity_type = 2;
    repeated UsedEntity used = 3;
    repeated AgentAssociation was_associated_with = 4;
    google.protobuf.Timestamp started_at_time = 5;
    google.protobuf.Timestamp ended_at_time = 6;
    google.protobuf.Struct metadata = 7;
    ActivityEcoPrimals ecop = 8;
}

message UsedEntity {
    EntityReference entity = 1;
    string role = 2;
    google.protobuf.Timestamp time = 3;
}

message EntityReference {
    oneof reference {
        string braid_id = 1;
        string data_hash = 2;
        LoamEntryRef loam_entry = 3;
        string external_url = 4;
    }
    string mime_type = 5;
}

message AgentAssociation {
    string agent = 1;  // DID
    string role = 2;
    string on_behalf_of = 3;  // DID
}

message EcoPrimalsAttributes {
    string source_primal = 1;
    string niche = 2;
    string rhizo_session = 3;
    LoamCommitRef loam_commit = 4;
    string certificate_id = 5;
    CompressionMeta compression = 6;
}

message ActivityEcoPrimals {
    double compute_units = 1;
    uint64 storage_bytes = 2;
    uint64 network_bytes = 3;
    uint64 duration_ns = 4;
    string rhizo_session = 5;
    string toadstool_task = 6;
}

message CompressionMeta {
    uint64 vertex_count = 1;
    uint64 branch_count = 2;
    double ratio = 3;
    repeated string summarizes = 4;
}

message BraidSignature {
    string type = 1;
    google.protobuf.Timestamp created = 2;
    string verification_method = 3;
    string proof_purpose = 4;
    string proof_value = 5;
}

message LoamAnchor {
    string spine_id = 1;
    bytes entry_hash = 2;
    uint64 index = 3;
    google.protobuf.Timestamp anchored_at = 4;
    bool verified = 5;
}

message LoamEntryRef {
    string spine_id = 1;
    bytes entry_hash = 2;
}

message LoamCommitRef {
    string spine_id = 1;
    bytes entry_hash = 2;
    uint64 index = 3;
}

// ==================== Requests/Responses ====================

message CreateBraidRequest {
    string data_hash = 1;
    string mime_type = 2;
    uint64 size = 3;
    Activity activity = 4;
    repeated EntityReference derived_from = 5;
    google.protobuf.Struct metadata = 6;
    EcoPrimalsAttributes ecop = 7;
    bool anchor = 8;  // Anchor to LoamSpine immediately
    string anchor_spine = 9;  // Spine to anchor to
}

message CreateBraidResponse {
    Braid braid = 1;
    LoamAnchor anchor = 2;  // If anchored
}

message GetBraidRequest {
    string braid_id = 1;
}

message GetBraidResponse {
    Braid braid = 1;
}

message GetBraidsForDataRequest {
    string data_hash = 1;
    uint32 limit = 2;
}

message GetBraidsForDataResponse {
    repeated Braid braids = 1;
}

message ListBraidsRequest {
    BraidFilter filter = 1;
    Pagination pagination = 2;
}

message BraidFilter {
    string attributed_to = 1;  // DID
    string activity_type = 2;
    google.protobuf.Timestamp created_after = 3;
    google.protobuf.Timestamp created_before = 4;
    string niche = 5;
    bool anchored_only = 6;
}

message Pagination {
    uint32 limit = 1;
    string cursor = 2;
}

message ListBraidsResponse {
    repeated Braid braids = 1;
    string next_cursor = 2;
    uint64 total_count = 3;
}

message GetProvenanceGraphRequest {
    EntityReference entity = 1;
    uint32 max_depth = 2;
    bool include_activities = 3;
}

message GetProvenanceGraphResponse {
    string root_braid_id = 1;
    repeated Braid braids = 2;
    repeated Activity activities = 3;
    repeated ProvenanceEdge edges = 4;
    uint32 depth = 5;
    GraphStats stats = 6;
}

message ProvenanceEdge {
    string from_id = 1;
    string to_id = 2;
    string edge_type = 3;
    string activity_id = 4;
}

message GraphStats {
    uint32 braid_count = 1;
    uint32 activity_count = 2;
    uint32 agent_count = 3;
    uint32 edge_count = 4;
    uint32 max_depth = 5;
}

message GetAttributionChainRequest {
    EntityReference entity = 1;
    AttributionConfig config = 2;
}

message AttributionConfig {
    uint32 max_depth = 1;
    double inheritance_decay = 2;
    double min_share_threshold = 3;
    bool include_resources = 4;
}

message GetAttributionChainResponse {
    AttributionChain chain = 1;
}

message AttributionChain {
    EntityReference entity = 1;
    string root_braid_id = 2;
    repeated ContributorShare contributors = 3;
    ResourceTotals resources = 4;
    uint32 depth = 5;
}

message ContributorShare {
    string agent = 1;  // DID
    string role = 2;
    double share = 3;
    bool direct = 4;
    uint32 inheritance_depth = 5;
    repeated string source_braids = 6;
    ResourceContribution resources = 7;
}

message ResourceTotals {
    double compute_units = 1;
    uint64 storage_bytes = 2;
    uint64 network_bytes = 3;
    uint64 duration_ns = 4;
}

message ResourceContribution {
    double compute_units = 1;
    uint64 storage_bytes = 2;
    uint64 network_bytes = 3;
    uint64 data_bytes = 4;
}

message GetTopContributorsRequest {
    EntityReference entity = 1;
    uint32 limit = 2;
}

message GetTopContributorsResponse {
    repeated ContributorShare contributors = 1;
}

message GetAgentContributionsRequest {
    string agent = 1;  // DID
    google.protobuf.Timestamp start_time = 2;
    google.protobuf.Timestamp end_time = 3;
    Pagination pagination = 4;
}

message GetAgentContributionsResponse {
    string agent = 1;
    uint64 total_contributions = 2;
    double total_share_value = 3;
    repeated RoleContributions by_role = 4;
    repeated EntityContribution contributions = 5;
    string next_cursor = 6;
}

message RoleContributions {
    string role = 1;
    uint64 count = 2;
    double total_share = 3;
}

message EntityContribution {
    EntityReference entity = 1;
    string braid_id = 2;
    double share = 3;
    string role = 4;
    google.protobuf.Timestamp timestamp = 5;
}

message CompressSessionRequest {
    string session_id = 1;
    DehydrationSummary summary = 2;
    CompressionHint hint = 3;
}

message DehydrationSummary {
    string session_id = 1;
    string session_type = 2;
    bytes merkle_root = 3;
    uint64 vertex_count = 4;
    double compute_units = 5;
    google.protobuf.Timestamp started_at = 6;
    google.protobuf.Timestamp ended_at = 7;
}

enum CompressionHint {
    COMPRESSION_HINT_AUTO = 0;
    COMPRESSION_HINT_SINGLE = 1;
    COMPRESSION_HINT_SPLIT = 2;
    COMPRESSION_HINT_EPHEMERAL = 3;
}

message CompressSessionResponse {
    CompressionResultType result_type = 1;
    repeated Braid braids = 2;
    Braid summary_braid = 3;
    string discard_reason = 4;
}

enum CompressionResultType {
    COMPRESSION_RESULT_NONE = 0;
    COMPRESSION_RESULT_SINGLE = 1;
    COMPRESSION_RESULT_MULTIPLE = 2;
}

message CreateMetaBraidRequest {
    repeated string braid_ids = 1;
    string summary_type = 2;
}

message CreateMetaBraidResponse {
    Braid meta_braid = 1;
}

message AnchorBraidRequest {
    string braid_id = 1;
    string spine_id = 2;
}

message AnchorBraidResponse {
    LoamAnchor anchor = 1;
}

message VerifyAnchorRequest {
    string braid_id = 1;
}

message VerifyAnchorResponse {
    bool anchored = 1;
    LoamAnchor anchor = 2;
    bool verified = 3;
}

message SearchBraidsRequest {
    string query = 1;
    BraidFilter filter = 2;
    uint32 limit = 3;
}

message SearchBraidsResponse {
    repeated BraidSearchResult results = 1;
}

message BraidSearchResult {
    Braid braid = 1;
    double score = 2;
    repeated string highlights = 3;
}

message SubscribeBraidsRequest {
    BraidFilter filter = 1;
}

message GraphQLRequest {
    string query = 1;
    string operation_name = 2;
    google.protobuf.Struct variables = 3;
}

message GraphQLResponse {
    google.protobuf.Struct data = 1;
    repeated GraphQLError errors = 2;
}

message GraphQLError {
    string message = 1;
    repeated GraphQLLocation locations = 2;
    repeated string path = 3;
}

message GraphQLLocation {
    uint32 line = 1;
    uint32 column = 2;
}

message HealthRequest {}

message HealthResponse {
    string status = 1;
    string version = 2;
    google.protobuf.Timestamp timestamp = 3;
}
```

---

## 3. REST API

### 3.1 OpenAPI Specification

```yaml
openapi: 3.0.3
info:
  title: SweetGrass API
  description: Semantic Provenance & Attribution Layer
  version: 1.0.0
  license:
    name: AGPL-3.0

servers:
  - url: /api/v1

paths:
  /braids:
    post:
      summary: Create a new Braid
      operationId: createBraid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateBraidRequest'
      responses:
        '201':
          description: Braid created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Braid'
    get:
      summary: List Braids
      operationId: listBraids
      parameters:
        - name: attributed_to
          in: query
          schema:
            type: string
        - name: activity_type
          in: query
          schema:
            type: string
        - name: created_after
          in: query
          schema:
            type: string
            format: date-time
        - name: created_before
          in: query
          schema:
            type: string
            format: date-time
        - name: limit
          in: query
          schema:
            type: integer
            default: 20
        - name: cursor
          in: query
          schema:
            type: string
      responses:
        '200':
          description: Braids list
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BraidList'

  /braids/{braid_id}:
    get:
      summary: Get Braid by ID
      operationId: getBraid
      parameters:
        - name: braid_id
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Braid details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Braid'
            application/ld+json:
              schema:
                $ref: '#/components/schemas/BraidJsonLd'
        '404':
          description: Braid not found

  /braids/by-hash/{data_hash}:
    get:
      summary: Get Braids for a data hash
      operationId: getBraidsByHash
      parameters:
        - name: data_hash
          in: path
          required: true
          schema:
            type: string
        - name: limit
          in: query
          schema:
            type: integer
            default: 10
      responses:
        '200':
          description: Braids for hash
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Braid'

  /braids/{braid_id}/provenance:
    get:
      summary: Get provenance graph
      operationId: getProvenanceGraph
      parameters:
        - name: braid_id
          in: path
          required: true
          schema:
            type: string
        - name: depth
          in: query
          schema:
            type: integer
            default: 5
      responses:
        '200':
          description: Provenance graph
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ProvenanceGraph'

  /braids/{braid_id}/attribution:
    get:
      summary: Get attribution chain
      operationId: getAttributionChain
      parameters:
        - name: braid_id
          in: path
          required: true
          schema:
            type: string
        - name: max_depth
          in: query
          schema:
            type: integer
            default: 10
        - name: inheritance_decay
          in: query
          schema:
            type: number
            default: 0.7
      responses:
        '200':
          description: Attribution chain
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AttributionChain'

  /braids/{braid_id}/anchor:
    post:
      summary: Anchor Braid to LoamSpine
      operationId: anchorBraid
      parameters:
        - name: braid_id
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                spine_id:
                  type: string
              required:
                - spine_id
      responses:
        '200':
          description: Anchor created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/LoamAnchor'

  /agents/{agent_did}/contributions:
    get:
      summary: Get agent's contributions
      operationId: getAgentContributions
      parameters:
        - name: agent_did
          in: path
          required: true
          schema:
            type: string
        - name: start_time
          in: query
          schema:
            type: string
            format: date-time
        - name: end_time
          in: query
          schema:
            type: string
            format: date-time
        - name: limit
          in: query
          schema:
            type: integer
            default: 20
        - name: cursor
          in: query
          schema:
            type: string
      responses:
        '200':
          description: Agent contributions
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentContributions'

  /search:
    get:
      summary: Search Braids
      operationId: searchBraids
      parameters:
        - name: q
          in: query
          required: true
          schema:
            type: string
        - name: limit
          in: query
          schema:
            type: integer
            default: 10
      responses:
        '200':
          description: Search results
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/SearchResult'

  /compress:
    post:
      summary: Compress session to Braids
      operationId: compressSession
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CompressSessionRequest'
      responses:
        '200':
          description: Compression result
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CompressionResult'

  /graphql:
    post:
      summary: GraphQL endpoint
      operationId: graphql
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                query:
                  type: string
                operationName:
                  type: string
                variables:
                  type: object
              required:
                - query
      responses:
        '200':
          description: GraphQL response
          content:
            application/json:
              schema:
                type: object

  /health:
    get:
      summary: Health check
      operationId: health
      responses:
        '200':
          description: Health status
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HealthResponse'

components:
  schemas:
    Braid:
      type: object
      properties:
        id:
          type: string
        braid_type:
          type: string
        data_hash:
          type: string
        mime_type:
          type: string
        size:
          type: integer
        was_generated_by:
          $ref: '#/components/schemas/Activity'
        was_derived_from:
          type: array
          items:
            $ref: '#/components/schemas/EntityReference'
        was_attributed_to:
          type: string
        generated_at_time:
          type: string
          format: date-time
        metadata:
          type: object
        ecop:
          $ref: '#/components/schemas/EcoPrimalsAttributes'
        signature:
          $ref: '#/components/schemas/BraidSignature'
        loam_anchor:
          $ref: '#/components/schemas/LoamAnchor'

    Activity:
      type: object
      properties:
        id:
          type: string
        activity_type:
          type: string
        used:
          type: array
          items:
            $ref: '#/components/schemas/UsedEntity'
        was_associated_with:
          type: array
          items:
            $ref: '#/components/schemas/AgentAssociation'
        started_at_time:
          type: string
          format: date-time
        ended_at_time:
          type: string
          format: date-time

    EntityReference:
      type: object
      properties:
        braid_id:
          type: string
        data_hash:
          type: string
        mime_type:
          type: string

    UsedEntity:
      type: object
      properties:
        entity:
          $ref: '#/components/schemas/EntityReference'
        role:
          type: string
        time:
          type: string
          format: date-time

    AgentAssociation:
      type: object
      properties:
        agent:
          type: string
        role:
          type: string
        on_behalf_of:
          type: string

    EcoPrimalsAttributes:
      type: object
      properties:
        source_primal:
          type: string
        niche:
          type: string
        rhizo_session:
          type: string
        loam_commit:
          $ref: '#/components/schemas/LoamCommitRef'

    BraidSignature:
      type: object
      properties:
        type:
          type: string
        created:
          type: string
          format: date-time
        verification_method:
          type: string
        proof_purpose:
          type: string
        proof_value:
          type: string

    LoamAnchor:
      type: object
      properties:
        spine_id:
          type: string
        entry_hash:
          type: string
        index:
          type: integer
        anchored_at:
          type: string
          format: date-time
        verified:
          type: boolean

    LoamCommitRef:
      type: object
      properties:
        spine_id:
          type: string
        entry_hash:
          type: string
        index:
          type: integer

    ProvenanceGraph:
      type: object
      properties:
        root_braid_id:
          type: string
        braids:
          type: array
          items:
            $ref: '#/components/schemas/Braid'
        edges:
          type: array
          items:
            $ref: '#/components/schemas/ProvenanceEdge'
        depth:
          type: integer

    ProvenanceEdge:
      type: object
      properties:
        from_id:
          type: string
        to_id:
          type: string
        edge_type:
          type: string

    AttributionChain:
      type: object
      properties:
        entity:
          $ref: '#/components/schemas/EntityReference'
        root_braid_id:
          type: string
        contributors:
          type: array
          items:
            $ref: '#/components/schemas/ContributorShare'
        resources:
          $ref: '#/components/schemas/ResourceTotals'
        depth:
          type: integer

    ContributorShare:
      type: object
      properties:
        agent:
          type: string
        role:
          type: string
        share:
          type: number
        direct:
          type: boolean
        inheritance_depth:
          type: integer

    ResourceTotals:
      type: object
      properties:
        compute_units:
          type: number
        storage_bytes:
          type: integer
        network_bytes:
          type: integer

    AgentContributions:
      type: object
      properties:
        agent:
          type: string
        total_contributions:
          type: integer
        total_share_value:
          type: number
        by_role:
          type: array
          items:
            $ref: '#/components/schemas/RoleContributions'

    RoleContributions:
      type: object
      properties:
        role:
          type: string
        count:
          type: integer
        total_share:
          type: number

    SearchResult:
      type: object
      properties:
        braid:
          $ref: '#/components/schemas/Braid'
        score:
          type: number
        highlights:
          type: array
          items:
            type: string

    CompressionResult:
      type: object
      properties:
        result_type:
          type: string
          enum: [none, single, multiple]
        braids:
          type: array
          items:
            $ref: '#/components/schemas/Braid'
        summary_braid:
          $ref: '#/components/schemas/Braid'
        discard_reason:
          type: string

    CreateBraidRequest:
      type: object
      required:
        - data_hash
        - mime_type
        - size
      properties:
        data_hash:
          type: string
        mime_type:
          type: string
        size:
          type: integer
        activity:
          $ref: '#/components/schemas/Activity'
        derived_from:
          type: array
          items:
            $ref: '#/components/schemas/EntityReference'
        metadata:
          type: object
        anchor:
          type: boolean
        anchor_spine:
          type: string

    CompressSessionRequest:
      type: object
      required:
        - session_id
        - summary
      properties:
        session_id:
          type: string
        summary:
          type: object
        hint:
          type: string
          enum: [auto, single, split, ephemeral]

    BraidList:
      type: object
      properties:
        braids:
          type: array
          items:
            $ref: '#/components/schemas/Braid'
        next_cursor:
          type: string
        total_count:
          type: integer

    HealthResponse:
      type: object
      properties:
        status:
          type: string
        version:
          type: string
        timestamp:
          type: string
          format: date-time
```

---

## 4. GraphQL Schema

```graphql
scalar DateTime
scalar JSON

type Query {
  # ==================== Braid Queries ====================
  
  """Get a Braid by ID"""
  braid(id: ID!): Braid
  
  """Get Braids for a data hash"""
  braidsForData(hash: String!, limit: Int): [Braid!]!
  
  """List Braids with filtering"""
  braids(filter: BraidFilter, pagination: Pagination): BraidConnection!
  
  # ==================== Provenance Queries ====================
  
  """Get provenance graph for an entity"""
  provenanceGraph(entity: ID!, depth: Int): ProvenanceGraph!
  
  """Get attribution chain"""
  attributionChain(entity: ID!, config: AttributionConfigInput): AttributionChain!
  
  """Get top contributors for an entity"""
  topContributors(entity: ID!, limit: Int): [ContributorShare!]!
  
  # ==================== Agent Queries ====================
  
  """Get agent by DID"""
  agent(did: ID!): Agent
  
  """Get agent's contributions"""
  agentContributions(
    agent: ID!
    timeRange: TimeRangeInput
    pagination: Pagination
  ): AgentContributions!
  
  # ==================== Search ====================
  
  """Search Braids by text"""
  searchBraids(query: String!, limit: Int): [SearchResult!]!
}

type Mutation {
  """Create a new Braid"""
  createBraid(input: CreateBraidInput!): Braid!
  
  """Anchor Braid to LoamSpine"""
  anchorBraid(braidId: ID!, spineId: ID!): LoamAnchor!
  
  """Compress session to Braids"""
  compressSession(input: CompressSessionInput!): CompressionResult!
  
  """Create meta-Braid (summary)"""
  createMetaBraid(braidIds: [ID!]!, summaryType: String!): Braid!
}

type Subscription {
  """Subscribe to new Braids"""
  braidCreated(filter: BraidFilter): Braid!
}

# ==================== Types ====================

type Braid {
  id: ID!
  braidType: String!
  dataHash: String!
  mimeType: String!
  size: Int!
  wasGeneratedBy: Activity
  wasDerivedFrom: [Braid!]!
  derivedInto: [Braid!]!
  wasAttributedTo: Agent!
  generatedAtTime: DateTime!
  metadata: JSON
  ecop: EcoPrimalsAttributes
  signature: BraidSignature!
  loamAnchor: LoamAnchor
}

type Activity {
  id: ID!
  activityType: String!
  used: [UsedEntity!]!
  wasAssociatedWith: [AgentAssociation!]!
  startedAtTime: DateTime!
  endedAtTime: DateTime
  metadata: JSON
  ecop: ActivityEcoPrimals
}

type UsedEntity {
  entity: EntityRef!
  role: String!
  time: DateTime
}

type EntityRef {
  braidId: ID
  dataHash: String
  mimeType: String
}

type AgentAssociation {
  agent: Agent!
  role: String!
  onBehalfOf: Agent
}

type Agent {
  did: ID!
  agentType: String!
  name: String
  braidsCreated(limit: Int): [Braid!]!
  activitiesPerformed(limit: Int): [Activity!]!
  contributions: AgentContributions!
}

type EcoPrimalsAttributes {
  sourcePrimal: String
  niche: String
  rhizoSession: String
  loamCommit: LoamCommitRef
  certificateId: String
  compression: CompressionMeta
}

type ActivityEcoPrimals {
  computeUnits: Float
  storageBytes: Int
  networkBytes: Int
  durationNs: Int
  rhizoSession: String
  toadstoolTask: String
}

type CompressionMeta {
  vertexCount: Int!
  branchCount: Int!
  ratio: Float!
  summarizes: [ID!]!
}

type BraidSignature {
  type: String!
  created: DateTime!
  verificationMethod: String!
  proofPurpose: String!
  proofValue: String!
}

type LoamAnchor {
  spineId: ID!
  entryHash: String!
  index: Int!
  anchoredAt: DateTime!
  verified: Boolean!
}

type LoamCommitRef {
  spineId: ID!
  entryHash: String!
  index: Int!
}

type ProvenanceGraph {
  root: Braid!
  braids: [Braid!]!
  activities: [Activity!]!
  edges: [ProvenanceEdge!]!
  depth: Int!
  stats: GraphStats!
}

type ProvenanceEdge {
  from: ID!
  to: ID!
  edgeType: String!
  activity: Activity
}

type GraphStats {
  braidCount: Int!
  activityCount: Int!
  agentCount: Int!
  edgeCount: Int!
  maxDepth: Int!
}

type AttributionChain {
  entity: EntityRef!
  rootBraid: Braid!
  contributors: [ContributorShare!]!
  resources: ResourceTotals!
  depth: Int!
}

type ContributorShare {
  agent: Agent!
  role: String!
  share: Float!
  direct: Boolean!
  inheritanceDepth: Int!
  sourceBraids: [Braid!]!
  resources: ResourceContribution
}

type ResourceTotals {
  computeUnits: Float!
  storageBytes: Int!
  networkBytes: Int!
  durationNs: Int!
}

type ResourceContribution {
  computeUnits: Float
  storageBytes: Int
  networkBytes: Int
  dataBytes: Int
}

type AgentContributions {
  agent: Agent!
  totalContributions: Int!
  totalShareValue: Float!
  byRole: [RoleContributions!]!
  contributions(pagination: Pagination): [EntityContribution!]!
}

type RoleContributions {
  role: String!
  count: Int!
  totalShare: Float!
}

type EntityContribution {
  entity: EntityRef!
  braid: Braid!
  share: Float!
  role: String!
  timestamp: DateTime!
}

type SearchResult {
  braid: Braid!
  score: Float!
  highlights: [String!]!
}

type CompressionResult {
  resultType: CompressionResultType!
  braids: [Braid!]!
  summaryBraid: Braid
  discardReason: String
}

enum CompressionResultType {
  NONE
  SINGLE
  MULTIPLE
}

type BraidConnection {
  edges: [BraidEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type BraidEdge {
  node: Braid!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

# ==================== Inputs ====================

input BraidFilter {
  attributedTo: ID
  activityType: String
  createdAfter: DateTime
  createdBefore: DateTime
  niche: String
  anchoredOnly: Boolean
}

input Pagination {
  first: Int
  after: String
  last: Int
  before: String
}

input TimeRangeInput {
  start: DateTime
  end: DateTime
}

input AttributionConfigInput {
  maxDepth: Int
  inheritanceDecay: Float
  minShareThreshold: Float
  includeResources: Boolean
}

input CreateBraidInput {
  dataHash: String!
  mimeType: String!
  size: Int!
  activity: ActivityInput
  derivedFrom: [EntityRefInput!]
  metadata: JSON
  anchor: Boolean
  anchorSpine: ID
}

input ActivityInput {
  activityType: String!
  used: [UsedEntityInput!]
  wasAssociatedWith: [AgentAssociationInput!]
  startedAtTime: DateTime!
  endedAtTime: DateTime
  metadata: JSON
}

input UsedEntityInput {
  braidId: ID
  dataHash: String
  role: String!
}

input EntityRefInput {
  braidId: ID
  dataHash: String
}

input AgentAssociationInput {
  agent: ID!
  role: String!
  onBehalfOf: ID
}

input CompressSessionInput {
  sessionId: ID!
  summary: JSON!
  hint: CompressionHint
}

enum CompressionHint {
  AUTO
  SINGLE
  SPLIT
  EPHEMERAL
}
```

---

## 5. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [ATTRIBUTION_GRAPH.md](./ATTRIBUTION_GRAPH.md) — Attribution API details

---

*SweetGrass: APIs for provenance and attribution.*

