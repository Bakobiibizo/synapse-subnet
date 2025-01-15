# GUI Component Development Prompt

## Overview
Create a lightweight web interface using HTMX components for human interaction with the Synapse subnet.

## Requirements

### Core Functionality
- Component-based UI architecture
- Real-time updates via HTMX
- Dark/light theme support
- Responsive design

### Integration Points
- Connects to API for data operations
- Uses WebSocket for real-time updates
- Shares authentication with other components

### Technical Requirements
- Built using Askama for templating
- HTMX for dynamic updates
- Minimal JavaScript
- CSS framework (optional)

### Components
1. Layout Components:
   - Navigation
   - Dashboard
   - Status bars
   - Notifications

2. Registrar Components:
   - Module list
   - Registration form
   - Status display

3. Validator Components:
   - Status monitor
   - Configuration panel
   - Performance metrics

4. Miner Components:
   - Registration
   - Performance tracking
   - Stake management

### State Management
- Server-side state
- HTMX for updates
- WebSocket for real-time data

## User Experience
- Fast initial load
- Smooth transitions
- Responsive feedback
- Error handling

## Dependencies
- askama
- axum
- htmx-rs
- tower-http
- sqlx
