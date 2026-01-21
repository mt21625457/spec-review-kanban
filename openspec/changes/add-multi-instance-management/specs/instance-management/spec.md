## ADDED Requirements

> **Note**: In this specification, instance IDs like "inst-001" are simplified examples for readability. In the actual implementation, all instance IDs use UUID v4 format (e.g., "550e8400-e29b-41d4-a716-446655440000").

### Requirement: Instance Registry

aicodex MUST maintain a registry of all vibe-kanban instances in its database.

#### Scenario: Create new instance
- **WHEN** administrator requests to create a new instance with name "dev-team-1"
- **THEN** system generates a UUID v4 as instance ID
- **AND** system allocates an available port from the configured range
- **AND** system creates the instance data directory structure
- **AND** system persists the instance metadata to the database
- **AND** system returns the created instance details

#### Scenario: List all instances
- **WHEN** administrator requests to list all instances
- **THEN** system returns a list of all registered instances
- **AND** each instance includes: id, name, port, status, health status, created_at

#### Scenario: Get instance details
- **WHEN** administrator requests details for instance "inst-001"
- **THEN** system returns the full instance configuration
- **AND** includes AI agent configurations
- **AND** includes usage statistics
- **AND** includes list of assigned users

#### Scenario: Update instance configuration
- **WHEN** administrator requests to update instance "inst-001" with new name "dev-team-alpha"
- **AND** the instance exists
- **THEN** system updates the instance metadata in database
- **AND** system returns the updated instance details

#### Scenario: Delete instance
- **WHEN** administrator requests to delete instance "inst-001"
- **AND** the instance is stopped
- **AND** no users are assigned to the instance
- **THEN** system removes the instance from the registry
- **AND** system deletes the instance data directory
- **AND** system releases the allocated port

#### Scenario: Prevent delete instance with users
- **WHEN** administrator requests to delete instance "inst-001"
- **AND** users are still assigned to the instance
- **THEN** system rejects the request with error "Cannot delete instance with assigned users"

#### Scenario: Prevent delete running instance
- **WHEN** administrator requests to delete instance "inst-001"
- **AND** the instance is running
- **THEN** system rejects the request with error "Instance must be stopped before deletion"

#### Scenario: Non-admin access denied
- **WHEN** a user with role "user" requests to create, update, or delete an instance
- **THEN** system returns 403 with error "Administrator access required"


### Requirement: Instance Lifecycle Management

aicodex MUST manage the lifecycle of vibe-kanban instances as child processes.

#### Scenario: Start instance
- **WHEN** administrator requests to start instance "inst-001"
- **AND** the instance is stopped
- **THEN** system creates the process with appropriate environment variables
- **AND** system sets VIBE_KANBAN_DATA_DIR to the instance data directory
- **AND** system sets VIBE_KANBAN_PORT to the allocated port
- **AND** system waits for the health check to pass (timeout: 60 seconds)
- **AND** system updates instance status to "running"

#### Scenario: Start instance timeout
- **WHEN** administrator requests to start instance "inst-001"
- **AND** the instance process starts but health check does not pass within 60 seconds
- **THEN** system terminates the process
- **AND** system updates instance status to "error"
- **AND** system returns error "Instance failed to start: health check timeout"

#### Scenario: Start already running instance
- **WHEN** administrator requests to start instance "inst-001"
- **AND** the instance is already running
- **THEN** system returns success without creating a new process

#### Scenario: Stop instance gracefully
- **WHEN** administrator requests to stop instance "inst-001"
- **AND** the instance is running
- **THEN** system sends SIGTERM to the process
- **AND** system waits up to 30 seconds for graceful shutdown
- **AND** system updates instance status to "stopped"

#### Scenario: Force stop unresponsive instance
- **WHEN** administrator requests to stop instance "inst-001"
- **AND** the instance does not respond to SIGTERM within 30 seconds
- **THEN** system sends SIGKILL to the process
- **AND** system updates instance status to "stopped"

#### Scenario: Restart instance
- **WHEN** administrator requests to restart instance "inst-001"
- **THEN** system stops the instance
- **AND** system starts the instance
- **AND** system returns success when instance is healthy


### Requirement: Instance Health Monitoring

aicodex MUST continuously monitor the health of running instances with a check interval of 10 seconds.

#### Scenario: Health check success
- **WHEN** health check runs for instance "inst-001"
- **AND** the instance responds to HTTP health endpoint within 5 seconds
- **THEN** system updates health_status to "healthy"
- **AND** system updates last_health_check timestamp
- **AND** system resets failure counter to 0

#### Scenario: Health check failure
- **WHEN** health check runs for instance "inst-001"
- **AND** the instance does not respond within 5 seconds
- **THEN** system updates health_status to "unhealthy"
- **AND** system increments failure counter

#### Scenario: Auto-restart unhealthy instance
- **WHEN** instance "inst-001" fails 3 consecutive health checks
- **AND** auto-restart is enabled for the instance
- **THEN** system stops the instance process
- **AND** system starts a new instance process
- **AND** system logs the restart event

#### Scenario: Process crash detection
- **WHEN** instance process exits unexpectedly
- **THEN** system updates instance status to "error"
- **AND** system captures the exit code and stderr
- **AND** system triggers auto-restart if enabled


### Requirement: Instance Data Isolation

Each vibe-kanban instance MUST have isolated data storage.

#### Scenario: Data directory structure
- **WHEN** a new instance is created
- **THEN** system creates directory structure:
  - `{data_root}/{instance_id}/db/` for SQLite database
  - `{data_root}/{instance_id}/config/` for instance configuration
  - `{data_root}/{instance_id}/worktrees/` for Git worktrees
  - `{data_root}/{instance_id}/logs/` for instance logs
  - `{data_root}/{instance_id}/ai-agents/` for AI agent configurations

#### Scenario: Database isolation
- **WHEN** instance "inst-001" creates a project "my-project"
- **THEN** the project is stored in `{data_root}/inst-001/db/db.sqlite`
- **AND** the project is not visible to other instances

#### Scenario: Worktree isolation
- **WHEN** instance "inst-001" clones a repository
- **THEN** the worktree is created under `{data_root}/inst-001/worktrees/`
- **AND** the worktree is not accessible to other instances


### Requirement: Instance Configuration

Each instance MUST have independent configuration.

#### Scenario: Port configuration
- **WHEN** instance is created
- **THEN** system allocates a unique port from the configured range (default 18100-18199)
- **AND** the port is not used by any other instance

#### Scenario: Port allocation failure
- **WHEN** administrator requests to create a new instance
- **AND** all ports in the configured range are allocated
- **THEN** system rejects the request with error "No available ports in range 18100-18199"

#### Scenario: Port in use by external process
- **WHEN** instance "inst-001" is starting
- **AND** the allocated port is occupied by an external process
- **THEN** system attempts to allocate a different available port
- **AND** system updates the instance port configuration
- **AND** if no port available, returns error "Unable to find available port"

#### Scenario: Update instance configuration
- **WHEN** administrator updates instance "inst-001" configuration
- **THEN** system persists the new configuration
- **AND** system signals the instance to reload configuration if running


### Requirement: Multi-Instance Proxy Routing

aicodex proxy MUST route requests to the correct instance based on user session.

#### Scenario: Route based on user session
- **WHEN** authenticated user sends request to `/api/proxy/projects`
- **AND** user is assigned to instance "inst-001"
- **AND** instance "inst-001" is running
- **THEN** system forwards the request to `http://127.0.0.1:{inst-001-port}/api/projects`
- **AND** system returns the response from the instance

#### Scenario: Auto-start instance on request
- **WHEN** authenticated user sends request to `/api/proxy/projects`
- **AND** user is assigned to instance "inst-001"
- **AND** instance "inst-001" is stopped
- **AND** instance has auto_start enabled
- **THEN** system starts instance "inst-001"
- **AND** system waits for health check to pass
- **AND** system forwards the request to the instance

#### Scenario: Route without assigned instance
- **WHEN** authenticated user sends request to `/api/proxy/projects`
- **AND** user has no assigned instance
- **THEN** system returns 403 with error "No instance assigned. Please contact administrator."

#### Scenario: Unauthenticated request
- **WHEN** unauthenticated client sends request to `/api/proxy/projects`
- **THEN** system returns 401 with error "Authentication required"


### Requirement: User Management

aicodex MUST support user registration and management with multi-instance assignment.

#### Scenario: Create user
- **WHEN** administrator creates a new user with username "john"
- **THEN** system creates user record with hashed password
- **AND** user has no instance assigned initially
- **AND** user has no current_instance_id set
- **AND** user role defaults to "user"

#### Scenario: List users
- **WHEN** administrator requests to list all users
- **THEN** system returns list of all users
- **AND** each user includes: id, username, display_name, role, assigned_instances, current_instance_id, is_active

#### Scenario: Assign user to instance
- **WHEN** administrator assigns user "john" to instance "inst-001"
- **AND** instance "inst-001" exists
- **THEN** system creates assignment record in user_instance_assignments table
- **AND** if user has no current_instance_id, system sets it to "inst-001"
- **AND** user can now access the instance

#### Scenario: Assign user to multiple instances
- **WHEN** administrator assigns user "john" to instance "inst-001"
- **AND** administrator assigns user "john" to instance "inst-002"
- **THEN** user has access to both instances
- **AND** user can switch between instances

#### Scenario: Unassign user from one instance
- **WHEN** administrator unassigns user "john" from instance "inst-001"
- **AND** user is also assigned to instance "inst-002"
- **THEN** system removes the assignment for "inst-001"
- **AND** if user's current_instance_id was "inst-001", system sets it to "inst-002"
- **AND** user can still access "inst-002"

#### Scenario: Unassign user from all instances
- **WHEN** administrator unassigns user "john" from all instances
- **THEN** system removes all assignment records for the user
- **AND** system sets user's current_instance_id to null
- **AND** user cannot access any instance until reassigned

#### Scenario: Prevent over-assignment
- **WHEN** administrator assigns user "john" to instance "inst-001"
- **AND** instance "inst-001" has max_users limit of 5
- **AND** instance already has 5 users assigned
- **THEN** system rejects with error "Instance has reached maximum user limit"

#### Scenario: Delete user
- **WHEN** administrator deletes user "john"
- **THEN** system removes user from database
- **AND** system removes all user_instance_assignments for the user
- **AND** user sessions are invalidated


### Requirement: User Authentication

aicodex MUST authenticate users before allowing access.

#### Scenario: User login success
- **WHEN** user "john" submits correct credentials
- **AND** user is active
- **AND** user is assigned to at least one instance
- **THEN** system creates session token
- **AND** system returns list of assigned instances
- **AND** system returns current_instance_id
- **AND** system auto-starts current instance if stopped and auto_start enabled

#### Scenario: User login with multiple instances
- **WHEN** user "john" submits correct credentials
- **AND** user is assigned to instances "inst-001" and "inst-002"
- **THEN** system returns both instances in the response
- **AND** system returns user's current_instance_id (or first assigned if none set)

#### Scenario: User login without instance
- **WHEN** user "john" submits correct credentials
- **AND** user has no assigned instance
- **THEN** system returns error "No workspace assigned. Please contact administrator."

#### Scenario: User login with inactive account
- **WHEN** user "john" submits correct credentials
- **AND** user is_active is false
- **THEN** system returns error "Account is deactivated"

#### Scenario: User login with wrong password
- **WHEN** user "john" submits incorrect password
- **THEN** system returns error "Invalid username or password"
- **AND** system does not reveal whether username exists

#### Scenario: User logout
- **WHEN** authenticated user requests logout
- **THEN** system invalidates current session
- **AND** session token can no longer be used

#### Scenario: Session expired
- **WHEN** user sends request with expired session token
- **THEN** system returns 401 with error "Session expired"
- **AND** user must login again to obtain new session


### Requirement: Instance Switching

Users MUST be able to switch between their assigned instances.

#### Scenario: Switch to assigned instance
- **WHEN** user "john" requests to switch to instance "inst-002"
- **AND** user is assigned to instance "inst-002"
- **THEN** system updates user's current_instance_id to "inst-002"
- **AND** system auto-starts instance if stopped and auto_start enabled
- **AND** system returns instance information

#### Scenario: Switch to unassigned instance
- **WHEN** user "john" requests to switch to instance "inst-003"
- **AND** user is NOT assigned to instance "inst-003"
- **THEN** system returns 403 with error "You do not have access to this instance"

#### Scenario: Get current instance
- **WHEN** authenticated user requests current instance information
- **THEN** system returns the instance matching user's current_instance_id
- **AND** includes instance status and health information

#### Scenario: List user's instances
- **WHEN** authenticated user requests their instance list
- **THEN** system returns all instances the user is assigned to
- **AND** each instance includes: id, name, status, health_status
