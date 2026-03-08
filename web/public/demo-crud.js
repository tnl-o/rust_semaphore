/**
 * Semaphore UI - CRUD Demo Application
 * Полная реализация CRUD операций для всех сущностей
 */

// ============================================================================
// Configuration
// ============================================================================

const API_BASE = '/api';
let TOKEN = localStorage.getItem('semaphore_token');
let CURRENT_USER = null;
let CURRENT_PROJECT_ID = null;

// ============================================================================
// API Helper Functions
// ============================================================================

async function apiRequest(endpoint, options = {}) {
    const url = `${API_BASE}${endpoint}`;
    const headers = {
        'Content-Type': 'application/json',
        ...options.headers,
    };

    if (TOKEN) {
        headers['Authorization'] = `Bearer ${TOKEN}`;
    }

    try {
        const response = await fetch(url, {
            ...options,
            headers,
        });

        if (response.status === 401) {
            logout();
            throw new Error('Unauthorized');
        }

        const data = await response.json();

        if (!response.ok) {
            throw new Error(data.message || data.error || 'Request failed');
        }

        return data;
    } catch (error) {
        console.error('API Error:', error);
        throw error;
    }
}

// ============================================================================
// Authentication
// ============================================================================

async function login(username, password) {
    try {
        const data = await apiRequest('/auth/login', {
            method: 'POST',
            body: JSON.stringify({ username, password }),
        });

        TOKEN = data.token || data;
        localStorage.setItem('semaphore_token', TOKEN);
        
        // Получаем информацию о текущем пользователе
        await getCurrentUser();
        
        return true;
    } catch (error) {
        throw error;
    }
}

function logout() {
    TOKEN = null;
    CURRENT_USER = null;
    CURRENT_PROJECT_ID = null;
    localStorage.removeItem('semaphore_token');
    showView('login-view');
}

async function getCurrentUser() {
    try {
        CURRENT_USER = await apiRequest('/user');
        updateUserInfo();
        return CURRENT_USER;
    } catch (error) {
        console.error('Failed to get current user:', error);
        return null;
    }
}

function updateUserInfo() {
    const userInfo = document.getElementById('user-info');
    if (CURRENT_USER) {
        userInfo.textContent = `${CURRENT_USER.name} (${CURRENT_USER.username})`;
    }
}

// ============================================================================
// View Management
// ============================================================================

function showView(viewId) {
    document.querySelectorAll('.view').forEach(view => {
        view.classList.add('hidden');
        view.classList.remove('active');
    });
    
    const targetView = document.getElementById(viewId);
    if (targetView) {
        targetView.classList.remove('hidden');
        targetView.classList.add('active');
    }
}

function showPage(pageId) {
    document.querySelectorAll('.page').forEach(page => {
        page.classList.add('hidden');
        page.classList.remove('active');
    });

    const targetPage = document.getElementById(pageId);
    if (targetPage) {
        targetPage.classList.remove('hidden');
        targetPage.classList.add('active');
    }

    // Update nav links
    const pageName = pageId.replace('-page', '');
    document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.remove('active');
        if (link.dataset.page === pageName) {
            link.classList.add('active');
        }
    });

    // Load page data
    loadPageData(pageId);
}

// ============================================================================
// Toast Notifications
// ============================================================================

function showToast(message, type = 'info') {
    const container = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    
    const icons = {
        success: '✅',
        error: '❌',
        warning: '⚠️',
        info: 'ℹ️',
    };
    
    toast.innerHTML = `
        <span class="toast-icon">${icons[type] || icons.info}</span>
        <span class="toast-message">${message}</span>
        <button class="toast-close">&times;</button>
    `;
    
    toast.querySelector('.toast-close').addEventListener('click', () => {
        toast.remove();
    });
    
    container.appendChild(toast);
    
    // Auto remove after 5 seconds
    setTimeout(() => {
        if (toast.parentNode) {
            toast.remove();
        }
    }, 5000);
}

// ============================================================================
// Modal Management
// ============================================================================

function openModal(title, content) {
    const modal = document.getElementById('modal');
    document.getElementById('modal-title').textContent = title;
    document.getElementById('modal-body').innerHTML = content;
    modal.classList.remove('hidden');
}

function closeModal() {
    const modal = document.getElementById('modal');
    modal.classList.add('hidden');
}

// ============================================================================
// CRUD Operations
// ============================================================================

// --------------------------- Projects CRUD ---------------------------

async function loadProjects() {
    try {
        const projects = await apiRequest('/projects');
        renderProjects(projects);
        updateStats('projects', projects.length);
        document.getElementById('projects-count').textContent = projects.length;
        updateProjectFilters(projects);
        return projects;
    } catch (error) {
        showToast('Ошибка загрузки проектов: ' + error.message, 'error');
        return [];
    }
}

function renderProjects(projects) {
    const container = document.getElementById('projects-list');
    
    if (projects.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">📁</div>
                <h3>Нет проектов</h3>
                <p>Создайте первый проект, чтобы начать работу</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = projects.map(project => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">${escapeHtml(project.name)}</h3>
                <div class="card-actions">
                    <button class="card-btn" onclick="editProject(${project.id})">✏️</button>
                    <button class="card-btn delete" onclick="deleteProject(${project.id})">🗑️</button>
                </div>
            </div>
            <div class="card-body">
                <p><strong>ID:</strong> ${project.id}</p>
                <p><strong>Создан:</strong> ${formatDate(project.created)}</p>
                ${project.alert ? '<p><strong>🔔 Уведомления включены</strong></p>' : ''}
            </div>
            <div class="card-footer">
                <button class="btn btn-secondary btn-sm" onclick="selectProject(${project.id})">
                    Открыть
                </button>
            </div>
        </div>
    `).join('');
}

async function createProject(projectData) {
    try {
        const project = await apiRequest('/projects', {
            method: 'POST',
            body: JSON.stringify(projectData),
        });
        showToast('Проект успешно создан', 'success');
        await loadProjects();
        await loadDashboardStats();
        return project;
    } catch (error) {
        showToast('Ошибка создания проекта: ' + error.message, 'error');
        throw error;
    }
}

async function updateProject(projectId, projectData) {
    try {
        // Отправляем только изменённые поля (API теперь поддерживает partial update)
        await apiRequest(`/projects/${projectId}`, {
            method: 'PUT',
            body: JSON.stringify(projectData),
        });
        showToast('Проект успешно обновлен', 'success');
        await loadProjects();
    } catch (error) {
        showToast('Ошибка обновления проекта: ' + error.message, 'error');
        throw error;
    }
}

async function deleteProject(projectId) {
    if (!confirm('Вы уверены, что хотите удалить этот проект? Это действие нельзя отменить.')) {
        return;
    }
    
    try {
        await apiRequest(`/projects/${projectId}`, {
            method: 'DELETE',
        });
        showToast('Проект успешно удален', 'success');
        await loadProjects();
        await loadDashboardStats();
    } catch (error) {
        showToast('Ошибка удаления проекта: ' + error.message, 'error');
        throw error;
    }
}

// --------------------------- Templates CRUD ---------------------------

async function loadTemplates() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('templates-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра шаблонов</p>
            </div>
        `;
        return [];
    }
    
    try {
        const templates = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/templates`);
        renderTemplates(templates);
        updateStats('templates', templates.length);
        document.getElementById('templates-count').textContent = templates.length;
        return templates;
    } catch (error) {
        showToast('Ошибка загрузки шаблонов: ' + error.message, 'error');
        return [];
    }
}

function renderTemplates(templates) {
    const container = document.getElementById('templates-list');
    
    if (templates.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">📋</div>
                <h3>Нет шаблонов</h3>
                <p>Создайте первый шаблон для этого проекта</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = templates.map(template => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">${escapeHtml(template.name)}</h3>
                <div class="card-actions">
                    <button class="card-btn" onclick="editTemplate(${template.id})">✏️</button>
                    <button class="card-btn delete" onclick="deleteTemplate(${template.id})">🗑️</button>
                </div>
            </div>
            <div class="card-body">
                <p><strong>Playbook:</strong> ${escapeHtml(template.playbook)}</p>
                ${template.description ? `<p><strong>Описание:</strong> ${escapeHtml(template.description)}</p>` : ''}
                <p><strong>ID:</strong> ${template.id}</p>
            </div>
            <div class="card-footer">
                <button class="btn btn-success btn-sm" onclick="runTemplate(${template.id})">
                    ▶️ Запустить
                </button>
            </div>
        </div>
    `).join('');
}

async function createTemplate(templateData) {
    try {
        const template = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/templates`, {
            method: 'POST',
            body: JSON.stringify(templateData),
        });
        showToast('Шаблон успешно создан', 'success');
        await loadTemplates();
        await loadDashboardStats();
        return template;
    } catch (error) {
        showToast('Ошибка создания шаблона: ' + error.message, 'error');
        throw error;
    }
}

async function updateTemplate(templateId, templateData) {
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/templates/${templateId}`, {
            method: 'PUT',
            body: JSON.stringify(templateData),
        });
        showToast('Шаблон успешно обновлен', 'success');
        await loadTemplates();
    } catch (error) {
        showToast('Ошибка обновления шаблона: ' + error.message, 'error');
        throw error;
    }
}

async function deleteTemplate(templateId) {
    if (!confirm('Вы уверены, что хотите удалить этот шаблон?')) {
        return;
    }
    
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/templates/${templateId}`, {
            method: 'DELETE',
        });
        showToast('Шаблон успешно удален', 'success');
        await loadTemplates();
    } catch (error) {
        showToast('Ошибка удаления шаблона: ' + error.message, 'error');
        throw error;
    }
}

// --------------------------- Tasks CRUD ---------------------------

async function loadTasks() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('tasks-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра задач</p>
            </div>
        `;
        return [];
    }
    
    try {
        const tasks = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/tasks`);
        renderTasks(tasks);
        updateStats('tasks', tasks.length);
        document.getElementById('tasks-count').textContent = tasks.length;
        return tasks;
    } catch (error) {
        showToast('Ошибка загрузки задач: ' + error.message, 'error');
        return [];
    }
}

function renderTasks(tasks) {
    const container = document.getElementById('tasks-list');
    
    if (tasks.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">⚡</div>
                <h3>Нет задач</h3>
                <p>Запустите первую задачу</p>
            </div>
        `;
        return;
    }
    
    const statusLabels = {
        waiting: 'Ожидание',
        running: 'Выполняется',
        success: 'Успешно',
        failed: 'Ошибка',
    };
    
    container.innerHTML = tasks.map(task => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">Задача #${task.id}</h3>
                <span class="status-badge status-${task.status}">${statusLabels[task.status] || task.status}</span>
            </div>
            <div class="card-body">
                <p><strong>Шаблон:</strong> ${task.template_id}</p>
                <p><strong>Playbook:</strong> ${escapeHtml(task.playbook || '-')}</p>
                <p><strong>Создана:</strong> ${formatDate(task.created)}</p>
                ${task.message ? `<p><strong>Сообщение:</strong> ${escapeHtml(task.message)}</p>` : ''}
            </div>
            <div class="card-footer">
                <div class="card-actions">
                    <button class="card-btn" onclick="editTask(${task.id})">📋</button>
                </div>
            </div>
        </div>
    `).join('');
}

async function createTask(taskData) {
    try {
        const task = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/tasks`, {
            method: 'POST',
            body: JSON.stringify(taskData),
        });
        showToast('Задача успешно создана', 'success');
        await loadTasks();
        await loadDashboardStats();
        return task;
    } catch (error) {
        showToast('Ошибка создания задачи: ' + error.message, 'error');
        throw error;
    }
}

window.editTask = async function(id) {
    try {
        const task = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/tasks/${id}`);

        document.getElementById('modal-title').textContent = `Задача #${id}`;
        document.getElementById('modal-body').innerHTML = `
            <div class="task-details">
                <p><strong>Статус:</strong> <span class="status-badge status-${task.status}">${task.status}</span></p>
                <p><strong>Шаблон:</strong> ${task.template_id}</p>
                <p><strong>Playbook:</strong> ${escapeHtml(task.playbook || '-')}</p>
                <p><strong>Создана:</strong> ${formatDate(task.created)}</p>
                ${task.message ? `<p><strong>Сообщение:</strong> ${escapeHtml(task.message)}</p>` : ''}
                ${task.output ? `<pre style="background: #1e1e1e; color: #d4d4d4; padding: 15px; border-radius: 5px; overflow-x: auto;">${escapeHtml(task.output)}</pre>` : ''}
                <div class="form-actions" style="margin-top: 20px;">
                    <button type="button" class="btn btn-secondary modal-close">Закрыть</button>
                </div>
            </div>
        `;

        document.getElementById('modal').classList.remove('hidden');

        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });

    } catch (error) {
        showToast('Ошибка загрузки задачи: ' + error.message, 'error');
    }
};

// --------------------------- Inventory CRUD ---------------------------

async function loadInventory() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('inventory-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра инвентаря</p>
            </div>
        `;
        return [];
    }
    
    try {
        const inventory = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/inventories`);
        renderInventory(inventory);
        updateStats('inventory', inventory.length);
        document.getElementById('inventory-count').textContent = inventory.length;
        return inventory;
    } catch (error) {
        showToast('Ошибка загрузки инвентаря: ' + error.message, 'error');
        return [];
    }
}

function renderInventory(inventory) {
    const container = document.getElementById('inventory-list');
    
    if (inventory.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">🖥️</div>
                <h3>Нет инвентаря</h3>
                <p>Добавьте первый инвентарь</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = inventory.map(item => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">${escapeHtml(item.name)}</h3>
                <div class="card-actions">
                    <button class="card-btn" onclick="editInventory(${item.id})">✏️</button>
                    <button class="card-btn delete" onclick="deleteInventory(${item.id})">🗑️</button>
                </div>
            </div>
            <div class="card-body">
                <p><strong>Тип:</strong> ${item.inventory_type || 'static'}</p>
                <p><strong>ID:</strong> ${item.id}</p>
            </div>
        </div>
    `).join('');
}

async function createInventory(inventoryData) {
    try {
        const item = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/inventories`, {
            method: 'POST',
            body: JSON.stringify(inventoryData),
        });
        showToast('Инвентарь успешно создан', 'success');
        await loadInventory();
        await loadDashboardStats();
        return item;
    } catch (error) {
        showToast('Ошибка создания инвентаря: ' + error.message, 'error');
        throw error;
    }
}

async function updateInventory(inventoryId, inventoryData) {
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/inventories/${inventoryId}`, {
            method: 'PUT',
            body: JSON.stringify(inventoryData),
        });
        showToast('Инвентарь успешно обновлен', 'success');
        await loadInventory();
    } catch (error) {
        showToast('Ошибка обновления инвентаря: ' + error.message, 'error');
        throw error;
    }
}

async function deleteInventory(inventoryId) {
    if (!confirm('Вы уверены, что хотите удалить этот инвентарь?')) {
        return;
    }
    
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/inventories/${inventoryId}`, {
            method: 'DELETE',
        });
        showToast('Инвентарь успешно удален', 'success');
        await loadInventory();
    } catch (error) {
        showToast('Ошибка удаления инвентаря: ' + error.message, 'error');
        throw error;
    }
}

// --------------------------- Repositories CRUD ---------------------------

async function loadRepositories() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('repositories-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра репозиториев</p>
            </div>
        `;
        return [];
    }
    
    try {
        const repos = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/repositories`);
        renderRepositories(repos);
        updateStats('repositories', repos.length);
        document.getElementById('repositories-count').textContent = repos.length;
        return repos;
    } catch (error) {
        showToast('Ошибка загрузки репозиториев: ' + error.message, 'error');
        return [];
    }
}

function renderRepositories(repositories) {
    const container = document.getElementById('repositories-list');
    
    if (repositories.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">📦</div>
                <h3>Нет репозиториев</h3>
                <p>Добавьте первый репозиторий</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = repositories.map(repo => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">${escapeHtml(repo.name)}</h3>
                <div class="card-actions">
                    <button class="card-btn" onclick="editRepository(${repo.id})">✏️</button>
                    <button class="card-btn delete" onclick="deleteRepository(${repo.id})">🗑️</button>
                </div>
            </div>
            <div class="card-body">
                <p><strong>URL:</strong> ${escapeHtml(repo.git_url)}</p>
                <p><strong>Ветвь:</strong> ${escapeHtml(repo.git_branch || 'main')}</p>
                <p><strong>ID:</strong> ${repo.id}</p>
            </div>
        </div>
    `).join('');
}

async function createRepository(repoData) {
    try {
        const repo = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/repositories`, {
            method: 'POST',
            body: JSON.stringify(repoData),
        });
        showToast('Репозиторий успешно создан', 'success');
        await loadRepositories();
        await loadDashboardStats();
        return repo;
    } catch (error) {
        showToast('Ошибка создания репозитория: ' + error.message, 'error');
        throw error;
    }
}

async function updateRepository(repoId, repoData) {
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/repositories/${repoId}`, {
            method: 'PUT',
            body: JSON.stringify(repoData),
        });
        showToast('Репозиторий успешно обновлен', 'success');
        await loadRepositories();
    } catch (error) {
        showToast('Ошибка обновления репозитория: ' + error.message, 'error');
        throw error;
    }
}

async function deleteRepository(repoId) {
    if (!confirm('Вы уверены, что хотите удалить этот репозиторий?')) {
        return;
    }
    
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/repositories/${repoId}`, {
            method: 'DELETE',
        });
        showToast('Репозиторий успешно удален', 'success');
        await loadRepositories();
    } catch (error) {
        showToast('Ошибка удаления репозитория: ' + error.message, 'error');
        throw error;
    }
}

// --------------------------- Environments CRUD ---------------------------

async function loadEnvironments() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('environments-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра окружений</p>
            </div>
        `;
        return [];
    }
    
    try {
        const envs = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/environments`);
        renderEnvironments(envs);
        updateStats('environments', envs.length);
        document.getElementById('environments-count').textContent = envs.length;
        return envs;
    } catch (error) {
        showToast('Ошибка загрузки окружений: ' + error.message, 'error');
        return [];
    }
}

function renderEnvironments(environments) {
    const container = document.getElementById('environments-list');
    
    if (environments.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">⚙️</div>
                <h3>Нет окружений</h3>
                <p>Добавьте первое окружение</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = environments.map(env => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">${escapeHtml(env.name)}</h3>
                <div class="card-actions">
                    <button class="card-btn" onclick="editEnvironment(${env.id})">✏️</button>
                    <button class="card-btn delete" onclick="deleteEnvironment(${env.id})">🗑️</button>
                </div>
            </div>
            <div class="card-body">
                <p><strong>ID:</strong> ${env.id}</p>
            </div>
        </div>
    `).join('');
}

async function createEnvironment(envData) {
    try {
        const env = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/environments`, {
            method: 'POST',
            body: JSON.stringify(envData),
        });
        showToast('Окружение успешно создано', 'success');
        await loadEnvironments();
        await loadDashboardStats();
        return env;
    } catch (error) {
        showToast('Ошибка создания окружения: ' + error.message, 'error');
        throw error;
    }
}

async function updateEnvironment(envId, envData) {
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/environments/${envId}`, {
            method: 'PUT',
            body: JSON.stringify(envData),
        });
        showToast('Окружение успешно обновлено', 'success');
        await loadEnvironments();
    } catch (error) {
        showToast('Ошибка обновления окружения: ' + error.message, 'error');
        throw error;
    }
}

async function deleteEnvironment(envId) {
    if (!confirm('Вы уверены, что хотите удалить это окружение?')) {
        return;
    }
    
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/environments/${envId}`, {
            method: 'DELETE',
        });
        showToast('Окружение успешно удалено', 'success');
        await loadEnvironments();
    } catch (error) {
        showToast('Ошибка удаления окружения: ' + error.message, 'error');
        throw error;
    }
}

// --------------------------- Access Keys CRUD ---------------------------

async function loadKeys() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('keys-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра ключей</p>
            </div>
        `;
        return [];
    }
    
    try {
        const keys = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/keys`);
        renderKeys(keys);
        updateStats('keys', keys.length);
        document.getElementById('keys-count').textContent = keys.length;
        return keys;
    } catch (error) {
        showToast('Ошибка загрузки ключей: ' + error.message, 'error');
        return [];
    }
}

function renderKeys(keys) {
    const container = document.getElementById('keys-list');
    
    if (keys.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">🔑</div>
                <h3>Нет ключей доступа</h3>
                <p>Добавьте первый ключ</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = keys.map(key => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">${escapeHtml(key.name)}</h3>
                <div class="card-actions">
                    <button class="card-btn" onclick="editKey(${key.id})">✏️</button>
                    <button class="card-btn delete" onclick="deleteKey(${key.id})">🗑️</button>
                </div>
            </div>
            <div class="card-body">
                <p><strong>Тип:</strong> ${key.type}</p>
                <p><strong>ID:</strong> ${key.id}</p>
            </div>
        </div>
    `).join('');
}

async function createKey(keyData) {
    try {
        const key = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/keys`, {
            method: 'POST',
            body: JSON.stringify(keyData),
        });
        showToast('Ключ успешно создан', 'success');
        await loadKeys();
        await loadDashboardStats();
        return key;
    } catch (error) {
        showToast('Ошибка создания ключа: ' + error.message, 'error');
        throw error;
    }
}

async function updateKey(keyId, keyData) {
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/keys/${keyId}`, {
            method: 'PUT',
            body: JSON.stringify(keyData),
        });
        showToast('Ключ успешно обновлен', 'success');
        await loadKeys();
    } catch (error) {
        showToast('Ошибка обновления ключа: ' + error.message, 'error');
        throw error;
    }
}

async function deleteKey(keyId) {
    if (!confirm('Вы уверены, что хотите удалить этот ключ?')) {
        return;
    }
    
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/keys/${keyId}`, {
            method: 'DELETE',
        });
        showToast('Ключ успешно удален', 'success');
        await loadKeys();
    } catch (error) {
        showToast('Ошибка удаления ключа: ' + error.message, 'error');
        throw error;
    }
}

// ============================================================================
// Dashboard & Stats
// ============================================================================

async function loadDashboardStats() {
    try {
        const projects = await apiRequest('/projects');
        
        let totalTemplates = 0;
        let totalTasks = 0;
        let totalInventory = 0;
        let totalRepositories = 0;
        let totalEnvironments = 0;
        let totalKeys = 0;
        let totalSchedules = 0;
        
        for (const project of projects) {
            try {
                const [templates, tasks, inventory, repos, envs, keys] = await Promise.all([
                    apiRequest(`/projects/${project.id}/templates`),
                    apiRequest(`/projects/${project.id}/tasks`),
                    apiRequest(`/projects/${project.id}/inventories`),
                    apiRequest(`/projects/${project.id}/repositories`),
                    apiRequest(`/projects/${project.id}/environments`),
                    apiRequest(`/projects/${project.id}/keys`),
                ]);
                
                totalTemplates += templates.length;
                totalTasks += tasks.length;
                totalInventory += inventory.length;
                totalRepositories += repos.length;
                totalEnvironments += envs.length;
                totalKeys += keys.length;
            } catch (e) {
                console.error('Error loading stats for project', project.id, e);
            }
        }
        
        updateStats('projects', projects.length);
        updateStats('templates', totalTemplates);
        updateStats('tasks', totalTasks);
        updateStats('inventory', totalInventory);
        updateStats('repositories', totalRepositories);
        updateStats('environments', totalEnvironments);
        updateStats('keys', totalKeys);
        updateStats('schedules', totalSchedules);
    } catch (error) {
        console.error('Failed to load dashboard stats:', error);
    }
}

function updateStats(entity, count) {
    const element = document.getElementById(`stat-${entity}`);
    if (element) {
        element.textContent = count;
    }
}

function updateProjectFilters(projects) {
    const filters = [
        'template-project-filter',
        'task-project-filter',
        'inventory-project-filter',
        'repository-project-filter',
        'environment-project-filter',
        'key-project-filter',
        'event-project-filter',
    ];
    
    filters.forEach(filterId => {
        const select = document.getElementById(filterId);
        if (select) {
            const currentValue = select.value;
            select.innerHTML = '<option value="">Все проекты</option>';
            projects.forEach(project => {
                const option = document.createElement('option');
                option.value = project.id;
                option.textContent = project.name;
                select.appendChild(option);
            });
            select.value = currentValue;
        }
    });
}

// ============================================================================
// Form Builders
// ============================================================================

function buildProjectForm(project = null) {
    return `
        <form id="project-form">
            <div class="form-group">
                <label for="project-name">Название *</label>
                <input type="text" id="project-name" name="name" required 
                       value="${escapeHtml(project?.name || '')}">
            </div>
            <div class="form-group">
                <label>
                    <input type="checkbox" name="alert" ${project?.alert ? 'checked' : ''}>
                    Включить уведомления
                </label>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Отмена</button>
                <button type="submit" class="btn btn-primary">Сохранить</button>
            </div>
        </form>
    `;
}

function buildTemplateForm(template = null) {
    return `
        <form id="template-form">
            <div class="form-group">
                <label for="template-name">Название *</label>
                <input type="text" id="template-name" name="name" required 
                       value="${escapeHtml(template?.name || '')}">
            </div>
            <div class="form-group">
                <label for="template-playbook">Playbook *</label>
                <input type="text" id="template-playbook" name="playbook" required 
                       value="${escapeHtml(template?.playbook || '')}">
            </div>
            <div class="form-group">
                <label for="template-description">Описание</label>
                <textarea id="template-description" name="description">${escapeHtml(template?.description || '')}</textarea>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModal()">Отмена</button>
                <button type="submit" class="btn btn-primary">Сохранить</button>
            </div>
        </form>
    `;
}

// ============================================================================
// Modal Actions
// ============================================================================

function showCreateProjectModal() {
    openModal('Создать проект', buildProjectForm());
    
    document.getElementById('project-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const formData = new FormData(e.target);
        const projectData = {
            name: formData.get('name'),
            alert: formData.get('alert') === 'on',
        };
        
        try {
            await createProject(projectData);
            closeModal();
        } catch (error) {
            // Error already shown via toast
        }
    });
}

function showCreateTemplateModal() {
    openModal('Создать шаблон', buildTemplateForm());
    
    document.getElementById('template-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const formData = new FormData(e.target);
        const templateData = {
            name: formData.get('name'),
            playbook: formData.get('playbook'),
            description: formData.get('description'),
        };
        
        try {
            await createTemplate(templateData);
            closeModal();
        } catch (error) {
            // Error already shown via toast
        }
    });
}

// ============================================================================
// Event Handlers & Navigation
// ============================================================================

function selectProject(projectId) {
    CURRENT_PROJECT_ID = projectId;
    
    // Загружаем все данные выбранного проекта
    loadTemplates();
    loadTasks();
    loadInventory();
    loadRepositories();
    loadEnvironments();
    loadKeys();
    loadSchedules();
    
    // Переключаем на вкладку шаблонов
    showPage('templates-page');
    
    // Показываем уведомление
    const project = window.CACHED_PROJECTS?.find(p => p.id === projectId);
    if (project) {
        showToast(`Открыт проект: ${project.name}`, 'success');
    }
}

function loadPageData(pageId) {
    switch (pageId) {
        case 'dashboard-page':
            loadDashboardStats();
            break;
        case 'projects-page':
            loadProjects();
            break;
        case 'templates-page':
            loadTemplates();
            break;
        case 'tasks-page':
            loadTasks();
            break;
        case 'inventory-page':
            loadInventory();
            break;
        case 'repositories-page':
            loadRepositories();
            break;
        case 'environments-page':
            loadEnvironments();
            break;
        case 'keys-page':
            loadKeys();
            break;
        case 'schedules-page':
            loadSchedules();
            break;
        case 'events-page':
            loadEvents();
            break;
    }
}

// ============================================================================
// Schedules CRUD
// ============================================================================

async function loadSchedules() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('schedules-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра расписаний</p>
            </div>
        `;
        return [];
    }
    
    try {
        const schedules = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/schedule`);
        renderSchedules(schedules);
        updateStats('schedules', schedules.length);
        document.getElementById('schedules-count').textContent = schedules.length;
        return schedules;
    } catch (error) {
        showToast('Ошибка загрузки расписаний: ' + error.message, 'error');
        return [];
    }
}

function renderSchedules(schedules) {
    const container = document.getElementById('schedules-list');
    
    if (schedules.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">🕐</div>
                <h3>Нет расписаний</h3>
                <p>Добавьте первое расписание</p>
            </div>
        `;
        return;
    }
    
    container.innerHTML = schedules.map(schedule => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">${escapeHtml(schedule.name)}</h3>
                <span class="status-badge ${schedule.active ? 'status-success' : 'status-waiting'}">
                    ${schedule.active ? 'Активно' : 'Неактивно'}
                </span>
            </div>
            <div class="card-body">
                <p><strong>Cron:</strong> <code>${escapeHtml(schedule.cron)}</code></p>
                <p><strong>Шаблон:</strong> ${schedule.template_id}</p>
                <p><strong>ID:</strong> ${schedule.id}</p>
            </div>
            <div class="card-footer">
                <div class="card-actions">
                    <button class="card-btn" onclick="editSchedule(${schedule.id})">✏️</button>
                    <button class="card-btn delete" onclick="deleteSchedule(${schedule.id})">🗑️</button>
                </div>
            </div>
        </div>
    `).join('');
}

async function createSchedule(scheduleData) {
    try {
        const schedule = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/schedule`, {
            method: 'POST',
            body: JSON.stringify(scheduleData),
        });
        showToast('Расписание успешно создано', 'success');
        await loadSchedules();
        await loadDashboardStats();
        return schedule;
    } catch (error) {
        showToast('Ошибка создания расписания: ' + error.message, 'error');
        throw error;
    }
}

async function updateSchedule(scheduleId, scheduleData) {
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/schedule/${scheduleId}`, {
            method: 'PUT',
            body: JSON.stringify(scheduleData),
        });
        showToast('Расписание успешно обновлено', 'success');
        await loadSchedules();
    } catch (error) {
        showToast('Ошибка обновления расписания: ' + error.message, 'error');
        throw error;
    }
}

async function deleteSchedule(scheduleId) {
    if (!confirm('Вы уверены, что хотите удалить это расписание?')) {
        return;
    }
    
    try {
        await apiRequest(`/projects/${CURRENT_PROJECT_ID}/schedule/${scheduleId}`, {
            method: 'DELETE',
        });
        showToast('Расписание успешно удалено', 'success');
        await loadSchedules();
    } catch (error) {
        showToast('Ошибка удаления расписания: ' + error.message, 'error');
        throw error;
    }
}

// ============================================================================
// Events (заглушка для будущего расширения)
// ============================================================================

async function loadEvents() {
    if (!CURRENT_PROJECT_ID) {
        document.getElementById('events-list').innerHTML = `
            <div class="empty-state">
                <p>Выберите проект для просмотра событий</p>
            </div>
        `;
        return [];
    }

    try {
        const events = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/events`);
        renderEvents(events);
        document.getElementById('events-count').textContent = events.length;
        return events;
    } catch (error) {
        showToast('Ошибка загрузки событий: ' + error.message, 'error');
        return [];
    }
}

function renderEvents(events) {
    const container = document.getElementById('events-list');

    if (events.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">📝</div>
                <h3>Нет событий</h3>
            </div>
        `;
        return;
    }

    container.innerHTML = events.map(event => `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">Событие #${event.id}</h3>
            </div>
            <div class="card-body">
                <p><strong>Объект:</strong> ${escapeHtml(event.object_type || '-')}</p>
                <p><strong>Действие:</strong> ${escapeHtml(event.action || '-')}</p>
                <p><strong>Дата:</strong> ${formatDate(event.created)}</p>
            </div>
        </div>
    `).join('');
}

// ============================================================================
// Utility Functions
// ============================================================================

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatDate(dateString) {
    if (!dateString) return '-';
    const date = new Date(dateString);
    return date.toLocaleDateString('ru-RU', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}

// ============================================================================
// Edit Functions (заглушки для будущего расширения)
// ============================================================================

// Эти функции будут реализованы аналогично create, но с загрузкой текущих данных
window.editProject = async function(id) {
    try {
        // Загружаем текущие данные проекта
        const project = await apiRequest(`/projects/${id}`);
        
        // Показываем модальное окно
        document.getElementById('modal-title').textContent = 'Редактировать проект';
        document.getElementById('modal-body').innerHTML = `
            <form id="edit-project-form">
                <input type="hidden" id="edit-project-id" value="${project.id}">
                <div class="form-group">
                    <label for="edit-project-name">Название</label>
                    <input type="text" id="edit-project-name" value="${escapeHtml(project.name)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-project-alert">Включить уведомления</label>
                    <input type="checkbox" id="edit-project-alert" ${project.alert ? 'checked' : ''}>
                </div>
                <div class="form-group">
                    <label for="edit-project-max-parallel">Макс. параллельных задач</label>
                    <input type="number" id="edit-project-max-parallel" value="${project.max_parallel_tasks}" min="0">
                </div>
                <div class="form-group">
                    <label for="edit-project-type">Тип</label>
                    <select id="edit-project-type">
                        <option value="default" ${project.type === 'default' ? 'selected' : ''}>Default</option>
                        <option value="terraform" ${project.type === 'terraform' ? 'selected' : ''}>Terraform</option>
                    </select>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary modal-close">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        // Показываем модальное окно
        document.getElementById('modal').classList.remove('hidden');
        
        // Обработчик сохранения
        document.getElementById('edit-project-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const updatedData = {
                name: document.getElementById('edit-project-name').value,
                alert: document.getElementById('edit-project-alert').checked,
                max_parallel_tasks: parseInt(document.getElementById('edit-project-max-parallel').value),
                type: document.getElementById('edit-project-type').value,
                alert_chat: null,
                default_secret_storage_id: null
            };
            
            await updateProject(id, updatedData);
            document.getElementById('modal').classList.add('hidden');
            await loadProjects();
        });
        
        // Закрытие модального окна
        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });
        
    } catch (error) {
        showToast('Ошибка загрузки проекта: ' + error.message, 'error');
    }
};

window.editTemplate = async function(id) {
    try {
        const template = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/templates/${id}`);
        
        document.getElementById('modal-title').textContent = 'Редактировать шаблон';
        document.getElementById('modal-body').innerHTML = `
            <form id="edit-template-form">
                <input type="hidden" id="edit-template-id" value="${template.id}">
                <div class="form-group">
                    <label for="edit-template-name">Название</label>
                    <input type="text" id="edit-template-name" value="${escapeHtml(template.name)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-template-playbook">Playbook</label>
                    <input type="text" id="edit-template-playbook" value="${escapeHtml(template.playbook)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-template-description">Описание</label>
                    <textarea id="edit-template-description" rows="3">${escapeHtml(template.description || '')}</textarea>
                </div>
                <div class="form-group">
                    <label for="edit-template-type">Тип</label>
                    <select id="edit-template-type">
                        <option value="ansible" ${template.type === 'ansible' ? 'selected' : ''}>Ansible</option>
                        <option value="terraform" ${template.type === 'terraform' ? 'selected' : ''}>Terraform</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="edit-template-app">App</label>
                    <select id="edit-template-app">
                        <option value="ansible" ${template.app === 'ansible' ? 'selected' : ''}>Ansible</option>
                        <option value="terraform" ${template.app === 'terraform' ? 'selected' : ''}>Terraform</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="edit-template-git-branch">Git ветка</label>
                    <input type="text" id="edit-template-git-branch" value="${escapeHtml(template.git_branch || 'main')}">
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary modal-close">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        document.getElementById('modal').classList.remove('hidden');
        
        document.getElementById('edit-template-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const updatedData = {
                name: document.getElementById('edit-template-name').value,
                playbook: document.getElementById('edit-template-playbook').value,
                description: document.getElementById('edit-template-description').value,
                type: document.getElementById('edit-template-type').value,
                app: document.getElementById('edit-template-app').value,
                git_branch: document.getElementById('edit-template-git-branch').value,
            };
            
            await updateTemplate(id, updatedData);
            document.getElementById('modal').classList.add('hidden');
            await loadTemplates();
        });
        
        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });
        
    } catch (error) {
        showToast('Ошибка загрузки шаблона: ' + error.message, 'error');
    }
};

window.editInventory = async function(id) {
    try {
        const inventory = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/inventories/${id}`);
        
        document.getElementById('modal-title').textContent = 'Редактировать инвентарь';
        document.getElementById('modal-body').innerHTML = `
            <form id="edit-inventory-form">
                <input type="hidden" id="edit-inventory-id" value="${inventory.id}">
                <div class="form-group">
                    <label for="edit-inventory-name">Название</label>
                    <input type="text" id="edit-inventory-name" value="${escapeHtml(inventory.name)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-inventory-type">Тип</label>
                    <select id="edit-inventory-type">
                        <option value="static" ${inventory.inventory_type === 'static' ? 'selected' : ''}>Static</option>
                        <option value="file" ${inventory.inventory_type === 'file' ? 'selected' : ''}>File</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="edit-inventory-data">Данные инвентаря</label>
                    <textarea id="edit-inventory-data" rows="8">${escapeHtml(inventory.inventory_data || '')}</textarea>
                </div>
                <div class="form-group">
                    <label for="edit-inventory-ssh-login">SSH логин</label>
                    <input type="text" id="edit-inventory-ssh-login" value="${escapeHtml(inventory.ssh_login || '')}">
                </div>
                <div class="form-group">
                    <label for="edit-inventory-ssh-port">SSH порт</label>
                    <input type="number" id="edit-inventory-ssh-port" value="${inventory.ssh_port || 22}" min="1" max="65535">
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary modal-close">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        document.getElementById('modal').classList.remove('hidden');
        
        document.getElementById('edit-inventory-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const updatedData = {
                name: document.getElementById('edit-inventory-name').value,
                inventory_type: document.getElementById('edit-inventory-type').value,
                inventory_data: document.getElementById('edit-inventory-data').value,
                ssh_login: document.getElementById('edit-inventory-ssh-login').value,
                ssh_port: parseInt(document.getElementById('edit-inventory-ssh-port').value),
            };
            
            await updateInventory(id, updatedData);
            document.getElementById('modal').classList.add('hidden');
            await loadInventories();
        });
        
        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });
        
    } catch (error) {
        showToast('Ошибка загрузки инвентаря: ' + error.message, 'error');
    }
};

window.editRepository = async function(id) {
    try {
        const repo = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/repositories/${id}`);
        
        document.getElementById('modal-title').textContent = 'Редактировать репозиторий';
        document.getElementById('modal-body').innerHTML = `
            <form id="edit-repository-form">
                <input type="hidden" id="edit-repository-id" value="${repo.id}">
                <div class="form-group">
                    <label for="edit-repository-name">Название</label>
                    <input type="text" id="edit-repository-name" value="${escapeHtml(repo.name)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-repository-url">Git URL</label>
                    <input type="url" id="edit-repository-url" value="${escapeHtml(repo.git_url)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-repository-type">Тип</label>
                    <select id="edit-repository-type">
                        <option value="git" ${repo.git_type === 'git' ? 'selected' : ''}>Git</option>
                        <option value="github" ${repo.git_type === 'github' ? 'selected' : ''}>GitHub</option>
                        <option value="gitlab" ${repo.git_type === 'gitlab' ? 'selected' : ''}>GitLab</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="edit-repository-branch">Ветка</label>
                    <input type="text" id="edit-repository-branch" value="${escapeHtml(repo.git_branch || 'main')}">
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary modal-close">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        document.getElementById('modal').classList.remove('hidden');
        
        document.getElementById('edit-repository-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const updatedData = {
                name: document.getElementById('edit-repository-name').value,
                git_url: document.getElementById('edit-repository-url').value,
                git_type: document.getElementById('edit-repository-type').value,
                git_branch: document.getElementById('edit-repository-branch').value,
            };
            
            await updateRepository(id, updatedData);
            document.getElementById('modal').classList.add('hidden');
            await loadRepositories();
        });
        
        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });
        
    } catch (error) {
        showToast('Ошибка загрузки репозитория: ' + error.message, 'error');
    }
};

window.editEnvironment = async function(id) {
    try {
        const env = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/environments/${id}`);
        
        document.getElementById('modal-title').textContent = 'Редактировать окружение';
        document.getElementById('modal-body').innerHTML = `
            <form id="edit-environment-form">
                <input type="hidden" id="edit-environment-id" value="${env.id}">
                <div class="form-group">
                    <label for="edit-environment-name">Название</label>
                    <input type="text" id="edit-environment-name" value="${escapeHtml(env.name)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-environment-json">JSON переменные</label>
                    <textarea id="edit-environment-json" rows="8">${escapeHtml(env.json || '{}')}</textarea>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary modal-close">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        document.getElementById('modal').classList.remove('hidden');
        
        document.getElementById('edit-environment-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const updatedData = {
                name: document.getElementById('edit-environment-name').value,
                json: document.getElementById('edit-environment-json').value,
            };
            
            await updateEnvironment(id, updatedData);
            document.getElementById('modal').classList.add('hidden');
            await loadEnvironments();
        });
        
        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });
        
    } catch (error) {
        showToast('Ошибка загрузки окружения: ' + error.message, 'error');
    }
};

window.editKey = async function(id) {
    try {
        const key = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/keys/${id}`);
        
        document.getElementById('modal-title').textContent = 'Редактировать ключ доступа';
        document.getElementById('modal-body').innerHTML = `
            <form id="edit-key-form">
                <input type="hidden" id="edit-key-id" value="${key.id}">
                <div class="form-group">
                    <label for="edit-key-name">Название</label>
                    <input type="text" id="edit-key-name" value="${escapeHtml(key.name)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-key-type">Тип</label>
                    <select id="edit-key-type">
                        <option value="ssh" ${key.type === 'ssh' ? 'selected' : ''}>SSH Key</option>
                        <option value="login_password" ${key.type === 'login_password' ? 'selected' : ''}>Login/Password</option>
                    </select>
                </div>
                <div class="form-group" id="edit-key-ssh-group">
                    <label for="edit-key-ssh">SSH Private Key</label>
                    <textarea id="edit-key-ssh" rows="6">${escapeHtml(key.ssh_key || '')}</textarea>
                </div>
                <div class="form-group" id="edit-key-login-group">
                    <label for="edit-key-login">Логин</label>
                    <input type="text" id="edit-key-login" value="${escapeHtml(key.login_password_login || '')}">
                </div>
                <div class="form-group" id="edit-key-password-group">
                    <label for="edit-key-password">Пароль</label>
                    <input type="password" id="edit-key-password" value="${escapeHtml(key.login_password_password || '')}">
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary modal-close">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        // Переключение видимости полей в зависимости от типа
        function toggleKeyFields() {
            const type = document.getElementById('edit-key-type').value;
            document.getElementById('edit-key-ssh-group').style.display = type === 'ssh' ? 'block' : 'none';
            document.getElementById('edit-key-login-group').style.display = type === 'login_password' ? 'block' : 'none';
            document.getElementById('edit-key-password-group').style.display = type === 'login_password' ? 'block' : 'none';
        }
        
        toggleKeyFields();
        document.getElementById('edit-key-type').addEventListener('change', toggleKeyFields);
        
        document.getElementById('modal').classList.remove('hidden');
        
        document.getElementById('edit-key-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const type = document.getElementById('edit-key-type').value;
            const updatedData = {
                name: document.getElementById('edit-key-name').value,
                type: type,
                ...(type === 'ssh' ? {
                    ssh_key: document.getElementById('edit-key-ssh').value,
                } : {
                    login_password_login: document.getElementById('edit-key-login').value,
                    login_password_password: document.getElementById('edit-key-password').value,
                })
            };
            
            await updateKey(id, updatedData);
            document.getElementById('modal').classList.add('hidden');
            await loadKeys();
        });
        
        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });
        
    } catch (error) {
        showToast('Ошибка загрузки ключа: ' + error.message, 'error');
    }
};

window.editSchedule = async function(id) {
    try {
        const schedule = await apiRequest(`/projects/${CURRENT_PROJECT_ID}/schedule/${id}`);
        
        document.getElementById('modal-title').textContent = 'Редактировать расписание';
        document.getElementById('modal-body').innerHTML = `
            <form id="edit-schedule-form">
                <input type="hidden" id="edit-schedule-id" value="${schedule.id}">
                <div class="form-group">
                    <label for="edit-schedule-name">Название</label>
                    <input type="text" id="edit-schedule-name" value="${escapeHtml(schedule.name)}" required>
                </div>
                <div class="form-group">
                    <label for="edit-schedule-cron">Cron выражение</label>
                    <input type="text" id="edit-schedule-cron" value="${escapeHtml(schedule.cron)}" placeholder="0 2 * * *" required>
                </div>
                <div class="form-group">
                    <label for="edit-schedule-active">Активно</label>
                    <input type="checkbox" id="edit-schedule-active" ${schedule.active ? 'checked' : ''}>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary modal-close">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        document.getElementById('modal').classList.remove('hidden');
        
        document.getElementById('edit-schedule-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const updatedData = {
                name: document.getElementById('edit-schedule-name').value,
                cron: document.getElementById('edit-schedule-cron').value,
                active: document.getElementById('edit-schedule-active').checked,
            };
            
            await updateSchedule(id, updatedData);
            document.getElementById('modal').classList.add('hidden');
            await loadSchedules();
        });
        
        document.querySelector('.modal-close').addEventListener('click', () => {
            document.getElementById('modal').classList.add('hidden');
        });
        
    } catch (error) {
        showToast('Ошибка загрузки расписания: ' + error.message, 'error');
    }
};

// ============================================================================
// Initialization
// ============================================================================

async function init() {
    // Login form handler
    document.getElementById('login-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        
        const username = document.getElementById('username').value;
        const password = document.getElementById('password').value;
        const errorDiv = document.getElementById('login-error');
        const submitBtn = e.target.querySelector('button[type="submit"]');
        
        try {
            submitBtn.disabled = true;
            submitBtn.querySelector('.btn-text').classList.add('hidden');
            submitBtn.querySelector('.btn-loader').classList.remove('hidden');
            errorDiv.classList.add('hidden');
            
            await login(username, password);
            showView('dashboard-view');
            await loadDashboardStats();
            showToast('Добро пожаловать!', 'success');
        } catch (error) {
            errorDiv.textContent = error.message || 'Ошибка входа';
            errorDiv.classList.remove('hidden');
        } finally {
            submitBtn.disabled = false;
            submitBtn.querySelector('.btn-text').classList.remove('hidden');
            submitBtn.querySelector('.btn-loader').classList.add('hidden');
        }
    });
    
    // Logout button
    document.getElementById('logout-btn').addEventListener('click', logout);
    
    // Navigation
    document.querySelectorAll('.nav-link').forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            const pageId = link.dataset.page + '-page';
            showPage(pageId);
        });
    });
    
    // Add buttons
    document.getElementById('add-project-btn')?.addEventListener('click', showCreateProjectModal);
    document.getElementById('add-template-btn')?.addEventListener('click', showCreateTemplateModal);
    document.getElementById('add-inventory-btn')?.addEventListener('click', showCreateInventoryModal);
    document.getElementById('add-repository-btn')?.addEventListener('click', showCreateRepositoryModal);
    document.getElementById('add-environment-btn')?.addEventListener('click', showCreateEnvironmentModal);
    document.getElementById('add-key-btn')?.addEventListener('click', showCreateKeyModal);
    document.getElementById('add-schedule-btn')?.addEventListener('click', showCreateScheduleModal);
    
    // Modal close handlers
    document.querySelector('.modal-close')?.addEventListener('click', closeModal);
    document.querySelector('.modal-overlay')?.addEventListener('click', closeModal);
    
    // Quick action buttons
    document.querySelectorAll('.action-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const action = btn.dataset.action;
            switch (action) {
                case 'create-project':
                    showCreateProjectModal();
                    break;
                case 'create-template':
                    showCreateTemplateModal();
                    break;
                case 'view-api':
                    window.open('/swagger', '_blank');
                    break;
            }
        });
    });
    
    // Check if already logged in
    if (TOKEN) {
        try {
            await getCurrentUser();
            if (CURRENT_USER) {
                showView('dashboard-view');
                await loadDashboardStats();
            }
        } catch (error) {
            logout();
        }
    }
}

// Start application
document.addEventListener('DOMContentLoaded', init);
