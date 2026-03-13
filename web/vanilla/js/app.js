/**
 * Semaphore Vanilla JS - Main Application
 * Точка входа приложения
 */

import Router from './router.js';
import Store from './store.js';
import api from './api.js';
import { $, $$, createElement, delegate } from './utils/dom.js';
import { alert, confirm } from './components/dialogs.js';
import DataTable from './components/tables.js';

// ==================== Global State ====================

const store = new Store({
  user: null,
  project: null,
  projects: [],
  systemInfo: null,
  sidebarOpen: true
});

// ==================== Router ====================

const routes = [
  { path: '/auth/login', handler: handleLogin },
  { path: '/auth/logout', handler: handleLogout },
  { path: '/', handler: handleDashboard },
  { path: '/projects', handler: handleProjects },
  { path: '/project/:projectId', redirect: '/project/:projectId/history' },
  { path: '/project/:projectId/history', handler: handleHistory },
  { path: '/project/:projectId/templates', handler: handleTemplates },
  { path: '/project/:projectId/inventory', handler: handleInventory },
  { path: '/project/:projectId/repositories', handler: handleRepositories },
  { path: '/project/:projectId/environment', handler: handleEnvironment },
  { path: '/project/:projectId/keys', handler: handleKeys },
  { path: '/project/:projectId/team', handler: handleTeam },
  { path: '/project/:projectId/schedule', handler: handleSchedule },
  { path: '/project/:projectId/integrations', handler: handleIntegrations },
  { path: '/project/:projectId/audit-log', handler: handleAuditLog },
  { path: '/project/:projectId/analytics', handler: handleAnalytics },
  { path: '/project/:projectId/settings', handler: handleSettings },
  { path: '/tasks', handler: handleTasks },
  { path: '/users', handler: handleUsers },
  { path: '/runners', handler: handleRunners },
  { path: '/apps', handler: handleApps },
  { path: '/tokens', handler: handleTokens },
  { path: '/404', handler: handleNotFound }
];

const router = new Router(routes);

// ==================== Page Handlers ====================

async function handleLogin() {
  // Если уже авторизован - редирект
  if (localStorage.getItem('semaphore_token')) {
    try {
      await api.getCurrentUser();
      window.location.href = '/';
      return;
    } catch (e) {
      localStorage.removeItem('semaphore_token');
    }
  }
  
  // Загружаем страницу логина
  const response = await fetch('/html/auth.html');
  const html = await response.text();
  document.body.innerHTML = html;
  
  // Re-init скрипты
  const script = document.createElement('script');
  script.type = 'module';
  script.src = '/js/auth.js';
  document.body.appendChild(script);
}

async function handleLogout() {
  try {
    await api.logout();
  } catch (e) {}
  localStorage.removeItem('semaphore_token');
  window.location.href = '/auth/login';
}

async function handleDashboard() {
  await loadLayout();
  
  const projects = await api.getProjects();
  store.state.projects = projects;
  
  const content = $('#page-content');
  if (!content) return;
  
  if (projects.length === 0) {
    content.innerHTML = `
      <div class="text-h4 mb-4">Добро пожаловать в Semaphore</div>
      <p class="mb-4">У вас пока нет проектов. Создайте первый проект, чтобы начать работу.</p>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-project-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Создать проект
      </button>
    `;
    
    $('#create-project-btn')?.addEventListener('click', () => {
      router.push('/project/new');
    });
  } else {
    content.innerHTML = `
      <div class="text-h4 mb-4">Проекты</div>
      <div class="v-row">
        ${projects.map(p => `
          <div class="v-col-4">
            <div class="v-card" style="padding: 16px; cursor: pointer;" data-project-id="${p.id}">
              <div class="text-h6 mb-2">${escapeHtml(p.name)}</div>
              <p class="text-body-2" style="color: rgba(0,0,0,0.6);">
                ${escapeHtml(p.description || 'Нет описания')}
              </p>
            </div>
          </div>
        `).join('')}
      </div>
    `;
    
    $$('.v-card[data-project-id]', content).forEach(card => {
      card.addEventListener('click', () => {
        router.push(`/project/${card.dataset.projectId}/history`);
      });
    });
  }
}

async function handleProjects() {
  await loadLayout();
  handleDashboard(); // То же самое
}

async function handleHistory(params) {
  await loadLayout(params.projectId);
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">История задач</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="run-task-btn">
        <i class="v-icon mdi mdi-play"></i>
        Запустить задачу
      </button>
    </div>
    <div id="tasks-table"></div>
  `;
  
  // Загрузка задач
  const tasks = await api.getTasks(params.projectId);
  
  const tableContainer = $('#tasks-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Шаблон', value: 'template_name' },
        { text: 'Статус', value: 'status', format: formatTaskStatus },
        { text: 'Дата', value: 'created', format: (v) => formatDate(v) },
        { text: 'Длительность', value: 'end', format: (v, item) => formatDuration(item.start, v) }
      ],
      data: tasks || [],
      onRowClick: (item) => {
        router.push(`/project/${params.projectId}/tasks/${item.id}`);
      }
    });
  }
}

async function handleTemplates(params) {
  await loadLayout(params.projectId);
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Шаблоны</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-template-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Создать шаблон
      </button>
    </div>
    <div id="templates-table"></div>
  `;
  
  const templates = await api.getTemplates(params.projectId);
  
  const tableContainer = $('#templates-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'Playbook', value: 'playbook' },
        { text: 'Окружение', value: 'environment_name' },
        { text: 'Инвентарь', value: 'inventory_name' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: templates || [],
      actions: [
        { 
          icon: 'mdi mdi-play', 
          tooltip: 'Запустить',
          handler: (item) => runTemplate(params.projectId, item.id)
        },
        { 
          icon: 'mdi mdi-pencil', 
          tooltip: 'Редактировать',
          handler: (item) => editTemplate(params.projectId, item.id)
        },
        { 
          icon: 'mdi mdi-delete', 
          tooltip: 'Удалить',
          handler: (item) => deleteTemplate(params.projectId, item.id)
        }
      ]
    });
  }
  
  $('#create-template-btn')?.addEventListener('click', () => {
    alert({ title: 'Создание шаблона', content: 'Функция в разработке' });
  });
}

async function handleInventory(params) {
  await loadLayout(params.projectId);
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Инвентари</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-inventory-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить инвентарь
      </button>
    </div>
    <div id="inventory-table"></div>
  `;
  
  const inventories = await api.getInventories(params.projectId);
  
  const tableContainer = $('#inventory-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'Тип', value: 'type', format: (v) => v === 'file' ? 'Файл' : 'Статический' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: inventories || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => editInventory(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteInventory(params.projectId, item.id) }
      ]
    });
  }
}

async function handleRepositories(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Репозитории</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить репозиторий
      </button>
    </div>
    <div id="repositories-table"></div>
  `;
  
  const repos = await api.getRepositories(params.projectId);
  
  const tableContainer = $('#repositories-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'URL', value: 'url' },
        { text: 'Ветка', value: 'branch' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: repos || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => editRepo(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteRepo(params.projectId, item.id) }
      ]
    });
  }
}

async function handleEnvironment(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Окружения</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить окружение
      </button>
    </div>
    <div id="environment-table"></div>
  `;
  
  const envs = await api.getEnvironments(params.projectId);
  
  const tableContainer = $('#environment-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'JSON', value: 'json', format: (v) => v ? 'Да' : 'Нет' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: envs || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => editEnv(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteEnv(params.projectId, item.id) }
      ]
    });
  }
}

async function handleKeys(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Ключи доступа</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить ключ
      </button>
    </div>
    <div id="keys-table"></div>
  `;
  
  const keys = await api.getKeys(params.projectId);
  
  const tableContainer = $('#keys-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'Тип', value: 'type' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: keys || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => editKey(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteKey(params.projectId, item.id) }
      ]
    });
  }
}

async function handleTeam(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Команда</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-account-plus"></i>
        Добавить участника
      </button>
    </div>
    <div id="team-table"></div>
  `;
  
  const team = await api.getTeam(params.projectId);
  
  const tableContainer = $('#team-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'user_id' },
        { text: 'Имя', value: 'username' },
        { text: 'Роль', value: 'role' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: team || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => editTeamMember(params.projectId, item.user_id) },
        { icon: 'mdi mdi-delete', handler: (item) => removeTeamMember(params.projectId, item.user_id) }
      ]
    });
  }
}

async function handleSchedule(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Расписание</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить расписание
      </button>
    </div>
    <div id="schedule-table"></div>
  `;
}

async function handleIntegrations(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Интеграции</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить интеграцию
      </button>
    </div>
    <div id="integrations-table"></div>
  `;
}

async function handleAuditLog(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Audit Log</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-filter"></i>
        Фильтры
      </button>
    </div>
    <div id="audit-log-table"></div>
  `;
  
  const logs = await api.getAuditLogs(params.projectId);
  
  const tableContainer = $('#audit-log-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Действие', value: 'action' },
        { text: 'Объект', value: 'object_name' },
        { text: 'Пользователь', value: 'username' },
        { text: 'Дата', value: 'created', format: (v) => formatDate(v) }
      ],
      data: logs?.records || []
    });
  }
}

async function handleAnalytics(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="text-h4 mb-4">Аналитика</div>
    <div class="v-row">
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Всего задач</div>
          <div class="text-h3">0</div>
        </div>
      </div>
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Успешных</div>
          <div class="text-h3" style="color: #4caf50;">0</div>
        </div>
      </div>
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Проваленных</div>
          <div class="text-h3" style="color: #f44336;">0</div>
        </div>
      </div>
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Процент успеха</div>
          <div class="text-h3">0%</div>
        </div>
      </div>
    </div>
  `;
}

async function handleSettings(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="text-h4 mb-4">Настройки проекта</div>
    <div class="v-card" style="padding: 24px; max-width: 600px;">
      <div class="v-text-field">
        <input type="text" id="project-name" placeholder=" ">
        <label for="project-name">Название проекта</label>
      </div>
      <div class="v-text-field">
        <textarea id="project-description" placeholder=" " style="width: 100%; min-height: 80px;"></textarea>
        <label for="project-description">Описание</label>
      </div>
      <button class="v-btn v-btn--contained v-btn--primary">Сохранить</button>
    </div>
  `;
}

async function handleTasks() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Все задачи</div>';
}

async function handleUsers() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Пользователи</div>';
}

async function handleRunners() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Раннеры</div>';
}

async function handleApps() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Приложения</div>';
}

async function handleTokens() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Токены API</div>';
}

async function handleNotFound() {
  await loadLayout();
  $('#page-content').innerHTML = `
    <div class="text-center" style="padding: 48px;">
      <div class="text-h1">404</div>
      <p class="text-h6">Страница не найдена</p>
      <button class="v-btn v-btn--contained v-btn--primary" onclick="history.back()">
        Назад
      </button>
    </div>
  `;
}

// ==================== Helper Functions ====================

async function loadLayout(projectId = null) {
  // Загружаем основной layout
  const response = await fetch('/html/index.html');
  const html = await response.text();
  document.body.innerHTML = html;
  
  // Инициализация layout
  initLayout(projectId);
  
  // Загрузка данных пользователя
  try {
    const user = await api.getCurrentUser();
    store.state.user = user;
    $('#username-display').textContent = user.username || user.name || 'Пользователь';
  } catch (e) {
    console.error('Failed to load user:', e);
  }
}

function initLayout(projectId) {
  // Toggle sidebar
  const menuToggle = $('#menu-toggle');
  const navDrawer = $('#nav-drawer');
  const mainContent = $('#main-content');
  
  menuToggle?.addEventListener('click', () => {
    store.state.sidebarOpen = !store.state.sidebarOpen;
    if (store.state.sidebarOpen) {
      navDrawer.style.display = '';
      mainContent.classList.remove('main-content--no-drawer');
    } else {
      navDrawer.style.display = 'none';
      mainContent.classList.add('main-content--no-drawer');
    }
  });
  
  // Logout
  $('#logout-btn')?.addEventListener('click', (e) => {
    e.preventDefault();
    handleLogout();
  });
  
  // Подсветка активного пункта меню
  const currentPath = window.location.pathname;
  $$('.v-list-item').forEach(item => {
    const route = item.dataset.route;
    if (route && currentPath.startsWith(route.replace(/:\w+/g, '\\w+'))) {
      item.classList.add('v-list-item--active');
    }
  });
}

function runTemplate(projectId, templateId) {
  alert({ 
    title: 'Запуск задачи', 
    content: `Запуск шаблона #${templateId}` 
  });
}

function editTemplate(projectId, templateId) {
  alert({ 
    title: 'Редактирование шаблона', 
    content: `Шаблон #${templateId}` 
  });
}

function deleteTemplate(projectId, templateId) {
  confirm({ 
    title: 'Удаление шаблона', 
    content: `Вы уверены, что хотите удалить шаблон #${templateId}?` 
  }).then((result) => {
    if (result) {
      showSnackbar('Шаблон удалён');
    }
  });
}

function editInventory(projectId, id) {
  alert({ title: 'Редактирование инвентаря', content: `Инвентарь #${id}` });
}

function deleteInventory(projectId, id) {
  confirm({ title: 'Удаление', content: `Удалить инвентарь #${id}?` });
}

function editRepo(projectId, id) {
  alert({ title: 'Редактирование', content: `Репозиторий #${id}` });
}

function deleteRepo(projectId, id) {
  confirm({ title: 'Удаление', content: `Удалить репозиторий #${id}?` });
}

function editEnv(projectId, id) {
  alert({ title: 'Редактирование', content: `Окружение #${id}` });
}

function deleteEnv(projectId, id) {
  confirm({ title: 'Удаление', content: `Удалить окружение #${id}?` });
}

function editKey(projectId, id) {
  alert({ title: 'Редактирование', content: `Ключ #${id}` });
}

function deleteKey(projectId, id) {
  confirm({ title: 'Удаление', content: `Удалить ключ #${id}?` });
}

function editTeamMember(projectId, userId) {
  alert({ title: 'Редактирование', content: `Участник #${userId}` });
}

function removeTeamMember(projectId, userId) {
  confirm({ title: 'Удаление', content: `Удалить участника #${userId}?` });
}

function showSnackbar(text, action = 'OK') {
  const snackbar = $('#snackbar');
  const snackbarText = $('#snackbar-text');
  const snackbarAction = $('#snackbar-action');
  
  if (snackbar && snackbarText) {
    snackbarText.textContent = text;
    snackbarAction.textContent = action;
    snackbar.style.display = 'flex';
    
    setTimeout(() => {
      snackbar.style.display = 'none';
    }, 3000);
  }
}

function formatTaskStatus(status) {
  const colors = {
    success: 'success',
    failed: 'error',
    running: 'info',
    waiting: 'warning'
  };
  const color = colors[status] || '';
  const labels = {
    success: 'Успешно',
    failed: 'Ошибка',
    running: 'Выполняется',
    waiting: 'Ожидание'
  };
  return `<span class="v-chip v-chip--${color}">${labels[status] || status}</span>`;
}

function formatDate(date) {
  if (!date) return '—';
  return new Date(date).toLocaleString('ru-RU', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });
}

function formatDuration(start, end) {
  if (!start) return '—';
  const s = new Date(start);
  const e = end ? new Date(end) : new Date();
  const diff = e - s;
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  
  if (hours > 0) {
    return `${hours}ч ${minutes % 60}м`;
  } else if (minutes > 0) {
    return `${minutes}м ${seconds % 60}с`;
  }
  return `${seconds}с`;
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// ==================== Init ====================

// Запуск приложения
(async () => {
  const token = localStorage.getItem('semaphore_token');
  if (token) {
    try {
      await api.getCurrentUser();
      router.loadRoute(window.location.pathname);
    } catch (e) {
      localStorage.removeItem('semaphore_token');
      router.loadRoute('/auth/login');
    }
  } else {
    router.loadRoute('/auth/login');
  }
})();
